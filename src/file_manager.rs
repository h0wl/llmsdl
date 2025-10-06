use std::path::{Path, PathBuf};
use std::fs;
use url::Url;
use crate::error::DownloadError;

/// Creates a local directory structure based on the domain from the URL
/// 
/// This function extracts the domain from the provided URL and creates
/// a local directory structure under the specified output directory.
/// 
/// # Arguments
/// * `url` - The base URL to extract the domain from
/// * `output_dir` - The base output directory where files should be stored
/// 
/// # Returns
/// * `Result<PathBuf, DownloadError>` - The path to the created directory
/// 
/// # Requirements
/// * 3.1: Create local directory named after source domain
pub fn create_local_directory(url: &str, output_dir: &str) -> Result<PathBuf, DownloadError> {
    // Parse the URL to extract the domain
    let parsed_url = Url::parse(url)
        .map_err(|e| DownloadError::InvalidUrl(format!("Failed to parse URL: {e}")))?;
    
    // Extract the host (domain) from the URL
    let host = parsed_url.host_str()
        .ok_or_else(|| DownloadError::InvalidUrl("URL must have a valid host".to_string()))?;
    
    // Create the domain string, including port if present
    let domain = if let Some(port) = parsed_url.port() {
        format!("{host}_{port}")
    } else {
        host.to_string()
    };
    
    // Sanitize the domain name for use as a directory name
    let sanitized_domain = sanitize_filename(&domain);
    
    // Create the base output directory path
    let base_dir = PathBuf::from(output_dir);
    let domain_dir = base_dir.join(sanitized_domain);
    
    // Create the directory structure if it doesn't exist
    fs::create_dir_all(&domain_dir)
        .map_err(DownloadError::IoError)?;
    
    Ok(domain_dir)
}

/// Generates a local file path for a given URL, preserving the directory structure
/// 
/// This function takes a URL and a base directory, then creates a local file path
/// that preserves the original directory structure from the URL.
/// 
/// # Arguments
/// * `url` - The full URL of the file to download
/// * `base_dir` - The base directory where files should be stored
/// 
/// # Returns
/// * `PathBuf` - The local file path where the file should be saved
/// 
/// # Requirements
/// * 3.2: Preserve relative path structure of downloaded files
/// * 3.3: Handle file path sanitization
pub fn get_local_file_path(url: &str, base_dir: &Path) -> Result<PathBuf, DownloadError> {
    // Parse the URL to extract the path
    let parsed_url = Url::parse(url)
        .map_err(|e| DownloadError::InvalidUrl(format!("Failed to parse URL: {e}")))?;
    
    // Get the path from the URL, removing the leading slash
    let url_path = parsed_url.path();
    let clean_path = if let Some(stripped) = url_path.strip_prefix('/') {
        stripped
    } else {
        url_path
    };
    
    // Handle empty path or root path
    if clean_path.is_empty() || clean_path == "/" {
        return Ok(base_dir.join("index.html"));
    }
    
    // Split the path into components and sanitize each part
    let path_components: Vec<String> = clean_path
        .split('/')
        .filter(|component| !component.is_empty())
        .map(sanitize_filename)
        .collect();
    
    // Build the local file path
    let mut local_path = base_dir.to_path_buf();
    for component in path_components {
        local_path = local_path.join(component);
    }
    
    // Ensure the parent directory exists
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent)
            .map_err(DownloadError::IoError)?;
    }
    
    Ok(local_path)
}

/// Sanitizes a filename by removing or replacing invalid characters
/// 
/// This function ensures that filenames are safe to use on the local filesystem
/// by replacing problematic characters with safe alternatives.
/// 
/// # Arguments
/// * `filename` - The original filename to sanitize
/// 
/// # Returns
/// * `String` - The sanitized filename
fn sanitize_filename(filename: &str) -> String {
    // Characters that are problematic on various filesystems
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
    
    let mut sanitized = String::new();
    
    for ch in filename.chars() {
        if invalid_chars.contains(&ch) {
            // Replace invalid characters with underscore
            sanitized.push('_');
        } else if ch.is_control() {
            // Replace control characters with underscore
            sanitized.push('_');
        } else {
            sanitized.push(ch);
        }
    }
    
    // Handle special cases
    if sanitized.is_empty() {
        sanitized = "unnamed".to_string();
    }
    
    // Trim leading/trailing dots and spaces (problematic on Windows)
    sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();
    
    // If the result is empty after trimming, use a default name
    if sanitized.is_empty() {
        sanitized = "unnamed".to_string();
    }
    
    // Limit length to avoid filesystem issues
    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }
    
    sanitized
}

