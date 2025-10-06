// Entry point and CLI setup

use clap::Parser;
use url::Url;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;

mod error;
mod http_client;
mod parser;
mod file_manager;

use error::{DownloadError, DownloadResult};
use http_client::HttpClient;
use parser::parse_llms_txt;
use file_manager::{create_local_directory, get_local_file_path};

/// A simple CLI tool to download documentation files from websites that implement the llms.txt standard
#[derive(Parser, Debug)]
#[command(name = "llmstxtdl")]
#[command(about = "Downloads documentation files from llms.txt enabled websites")]
#[command(version)]
struct Args {
    /// The base URL of the website to check for llms.txt
    #[arg(help = "Website URL (e.g., https://example.com)")]
    url: String,
    
    /// Output directory for downloaded documentation files
    #[arg(short = 'o', long = "output", help = "Output directory for downloaded files")]
    output: String,
    
    /// Number of concurrent download threads
    #[arg(short = 't', long = "threads", default_value = "5", help = "Number of concurrent download threads")]
    threads: usize,
}

#[tokio::main]
async fn main() -> Result<(), DownloadError> {
    let args = Args::parse();
    
    // Validate the URL format
    let validated_url = validate_url(&args.url)?;
    
    println!("🔍 Processing URL: {validated_url}");
    
    // Process the URL and download files
    match process_url(&validated_url, &args.output, args.threads).await {
        Ok(result) => {
            // Display final summary
            println!("\n✅ Process completed!");
            println!("{result}");
            
            if result.all_successful() {
                println!("🎉 All files downloaded successfully!");
            } else if result.success_count() > 0 {
                println!("⚠️  Some files failed to download, but {} files were successful.", result.success_count());
            } else {
                println!("❌ No files were downloaded successfully.");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {e}");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Main processing function that orchestrates the entire download workflow
async fn process_url(base_url: &str, output_dir: &str, max_concurrent: usize) -> Result<DownloadResult, DownloadError> {
    let client = HttpClient::new();
    let mut result = DownloadResult::new();
    
    // Step 1: Check for llms.txt file
    let llms_txt_url = format!("{base_url}/llms.txt");
    println!("🔍 Looking for llms.txt at: {llms_txt_url}");
    
    let llms_content = match client.fetch_content(&llms_txt_url).await {
        Ok(content) => {
            println!("✅ Found llms.txt file");
            content
        }
        Err(e) => {
            return Err(DownloadError::ParseError(format!(
                "Could not find or access llms.txt at {llms_txt_url}: {e}"
            )));
        }
    };
    
    // Step 2: Parse llms.txt content to get file URLs
    println!("📝 Parsing llms.txt content...");
    let file_urls = parse_llms_txt(&llms_content, base_url)?;
    
    if file_urls.is_empty() {
        println!("⚠️  No files found in llms.txt");
        return Ok(result);
    }
    
    println!("📋 Found {} files to download", file_urls.len());
    
    // Step 3: Create local directory structure
    println!("📁 Creating local directory structure...");
    let base_dir = create_local_directory(base_url, output_dir)?;
    println!("📁 Files will be saved to: {}", base_dir.display());
    
    // Step 4: Download files concurrently with enhanced progress reporting
    println!("\n🚀 Starting concurrent downloads with {} threads...", max_concurrent);
    
    // Create semaphore to limit concurrent downloads
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    
    // Create multi-progress for concurrent downloads
    let multi_progress = Arc::new(MultiProgress::new());
    
    // Create overall progress bar
    let overall_progress = multi_progress.add(ProgressBar::new(file_urls.len() as u64));
    overall_progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    overall_progress.set_message("Downloading files...");
    
    // Create download tasks for concurrent execution
    let total_files = file_urls.len();
    let download_tasks: Vec<_> = file_urls
        .into_iter()
        .enumerate()
        .map(|(index, file_url)| {
            let client = client.clone();
            let base_dir = base_dir.clone();
            let multi_progress = Arc::clone(&multi_progress);
            let overall_progress = overall_progress.clone();
            let semaphore = Arc::clone(&semaphore);
            
            tokio::spawn(async move {
                // Acquire semaphore permit to limit concurrency
                let _permit = semaphore.acquire().await.unwrap();
                
                let filename = file_url.split('/').next_back().unwrap_or(&file_url);
                
                // Create individual progress bar for this download
                let file_progress = multi_progress.add(ProgressBar::new_spinner());
                file_progress.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.blue} [{elapsed_precise}] {msg}")
                        .unwrap()
                );
                file_progress.set_message(format!("[{}/{}] {}", index + 1, total_files, filename));
                
                let result = download_single_file_with_progress(&client, &file_url, &base_dir, &file_progress).await;
                
                match &result {
                    Ok((_local_path, bytes)) => {
                        let size_str = crate::error::DownloadResult::format_bytes(*bytes);
                        file_progress.finish_with_message(format!("✅ {filename} - {size_str}"));
                    }
                    Err(e) => {
                        file_progress.finish_with_message(format!("❌ {filename} - {e}"));
                    }
                }
                
                overall_progress.inc(1);
                (file_url, result)
            })
        })
        .collect();
    
    // Wait for all downloads to complete
    let download_results = join_all(download_tasks).await;
    
    // Process results
    for task_result in download_results {
        match task_result {
            Ok((file_url, download_result)) => {
                match download_result {
                    Ok((local_path, bytes)) => {
                        result.add_success(file_url, local_path.display().to_string(), bytes);
                    }
                    Err(e) => {
                        result.add_failure(file_url, e.to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("Task error: {e}");
            }
        }
    }
    
    overall_progress.finish_with_message("All downloads completed");
    println!(); // Add spacing after progress bars
    
    Ok(result)
}



/// Downloads a single file with progress reporting for concurrent downloads
/// Returns the local path and number of bytes downloaded
async fn download_single_file_with_progress(
    client: &HttpClient,
    file_url: &str,
    base_dir: &std::path::Path,
    progress: &ProgressBar,
) -> Result<(std::path::PathBuf, u64), DownloadError> {
    // Determine the local file path
    let local_path = get_local_file_path(file_url, base_dir)?;
    
    // Update progress to show we're starting
    progress.set_message(format!("Starting download: {}", 
        file_url.split('/').next_back().unwrap_or(file_url)));
    
    // Download the file and get byte count
    let bytes = client.download_file(file_url, &local_path).await?;
    
    Ok((local_path, bytes))
}

/// Validates the provided URL and ensures it's properly formatted
fn validate_url(url_str: &str) -> Result<String, DownloadError> {
    // Parse the URL to validate its format
    let parsed_url = Url::parse(url_str)
        .map_err(|e| DownloadError::InvalidUrl(format!("Invalid URL format: {e}")))?;
    
    // Ensure the URL has a valid scheme (http or https)
    match parsed_url.scheme() {
        "http" | "https" => {},
        scheme => return Err(DownloadError::InvalidUrl(
            format!("Unsupported URL scheme '{scheme}'. Only http and https are supported.")
        )),
    }
    
    // Ensure the URL has a host
    if parsed_url.host().is_none() {
        return Err(DownloadError::InvalidUrl(
            "URL must have a valid host".to_string()
        ));
    }
    
    // Return the URL without trailing slash for consistency
    let mut clean_url = parsed_url.to_string();
    if clean_url.ends_with('/') {
        clean_url.pop();
    }
    
    Ok(clean_url)
}
