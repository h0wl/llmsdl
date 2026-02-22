use crate::error::{DownloadError, Result};
use reqwest::Client;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use indicatif::{ProgressBar, ProgressStyle};

/// HTTP client with retry logic and error handling
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
    base_delay: Duration,
}

/// User-Agent sent with requests. Many doc servers (e.g. Elastic) return 403 for
/// default library User-Agents; a recent Chrome UA is accepted.
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("default HTTP client");
        Self {
            client,
            max_retries: 3,
            base_delay: Duration::from_millis(500),
        }
    }

    /// Fetch text content from a URL with retry logic
    pub async fn fetch_content(&self, url: &str) -> Result<String> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.text().await {
                            Ok(content) => return Ok(content),
                            Err(e) => {
                                last_error = Some(DownloadError::NetworkError(e));
                            }
                        }
                    } else {
                        let status = response.status();
                        
                        // Create more specific error types
                        let error = match status.as_u16() {
                            404 => DownloadError::FileNotFound(url.to_string()),
                            403 => DownloadError::HttpError { status: 403, url: url.to_string() },
                            401 => DownloadError::HttpError { status: 401, url: url.to_string() },
                            500..=599 => DownloadError::HttpError { status: status.as_u16(), url: url.to_string() },
                            _ => DownloadError::HttpError { status: status.as_u16(), url: url.to_string() },
                        };
                        
                        // Don't retry on client errors (4xx)
                        if status.is_client_error() {
                            return Err(error);
                        }
                        
                        last_error = Some(error);
                    }
                }
                Err(e) => {
                    if e.is_timeout() {
                        last_error = Some(DownloadError::Timeout(url.to_string()));
                    } else {
                        last_error = Some(DownloadError::NetworkError(e));
                    }
                }
            }
            
            // Don't sleep after the last attempt
            if attempt < self.max_retries {
                let delay = self.calculate_delay(attempt);
                println!("⚠️  Request failed, retrying in {:?}... (attempt {}/{})", 
                        delay, attempt + 1, self.max_retries);
                sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            DownloadError::ParseError("Unknown network error".to_string())
        }))
    }

    /// Download a file from a URL and save it to the specified local path
    /// Returns the number of bytes downloaded
    pub async fn download_file(&self, url: &str, local_path: &Path) -> Result<u64> {
        let mut last_error = None;
        
        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // Get content length for progress bar
                        let content_length = response.content_length();
                        
                        // Create progress bar if we know the size
                        let progress_bar = if let Some(size) = content_length {
                            let pb = ProgressBar::new(size);
                            pb.set_style(
                                ProgressStyle::default_bar()
                                    .template("      [{bar:30.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                                    .unwrap()
                                    .progress_chars("#>-")
                            );
                            Some(pb)
                        } else {
                            None
                        };
                        
                        match response.bytes().await {
                            Ok(bytes) => {
                                let bytes_len = bytes.len() as u64;
                                
                                // Update progress bar
                                if let Some(pb) = &progress_bar {
                                    pb.set_position(bytes_len);
                                    pb.finish_and_clear();
                                }
                                
                                // Ensure the parent directory exists
                                if let Some(parent) = local_path.parent() {
                                    fs::create_dir_all(parent).await?;
                                }
                                
                                // Write the file
                                fs::write(local_path, bytes).await?;
                                return Ok(bytes_len);
                            }
                            Err(e) => {
                                if let Some(pb) = progress_bar {
                                    pb.abandon_with_message("Download failed");
                                }
                                last_error = Some(DownloadError::NetworkError(e));
                            }
                        }
                    } else {
                        let status = response.status();
                        
                        // Create more specific error types
                        let error = match status.as_u16() {
                            404 => DownloadError::FileNotFound(url.to_string()),
                            403 => DownloadError::HttpError { status: 403, url: url.to_string() },
                            401 => DownloadError::HttpError { status: 401, url: url.to_string() },
                            500..=599 => DownloadError::HttpError { status: status.as_u16(), url: url.to_string() },
                            _ => DownloadError::HttpError { status: status.as_u16(), url: url.to_string() },
                        };
                        
                        // Don't retry on client errors (4xx)
                        if status.is_client_error() {
                            return Err(error);
                        }
                        
                        last_error = Some(error);
                    }
                }
                Err(e) => {
                    if e.is_timeout() {
                        last_error = Some(DownloadError::Timeout(url.to_string()));
                    } else {
                        last_error = Some(DownloadError::NetworkError(e));
                    }
                }
            }
            
            // Don't sleep after the last attempt
            if attempt < self.max_retries {
                let delay = self.calculate_delay(attempt);
                println!("      ⚠️  Download failed, retrying in {:?}... (attempt {}/{})", 
                        delay, attempt + 1, self.max_retries);
                sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| {
            DownloadError::ParseError("Unknown download error".to_string())
        }))
    }

    /// Calculate exponential backoff delay
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let multiplier = 2_u64.pow(attempt);
        Duration::from_millis(self.base_delay.as_millis() as u64 * multiplier)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

