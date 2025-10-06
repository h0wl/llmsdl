use std::fmt;
use std::collections::HashMap;

/// Type alias for Results using our custom DownloadError
pub type Result<T> = std::result::Result<T, DownloadError>;

/// Custom error type for download operations
#[derive(Debug)]
pub enum DownloadError {
    /// Network-related errors from HTTP requests
    NetworkError(reqwest::Error),
    /// File I/O errors
    IoError(std::io::Error),
    /// URL parsing and validation errors
    InvalidUrl(String),
    /// Content parsing errors
    ParseError(String),
    /// HTTP status errors with specific status codes
    HttpError { status: u16, url: String },
    /// File not found (404) errors
    FileNotFound(String),

    /// Timeout errors
    Timeout(String),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::NetworkError(err) => {
                if err.is_timeout() {
                    write!(f, "Network timeout: The request took too long to complete. Try again later or check your internet connection.")
                } else if err.is_connect() {
                    write!(f, "Connection failed: Unable to connect to the server. Please check your internet connection and verify the server is accessible.")
                } else if err.is_request() {
                    write!(f, "Request error: Invalid request format or parameters. The URL may be malformed.")
                } else if err.is_decode() {
                    write!(f, "Content decode error: The server response could not be decoded properly.")
                } else {
                    write!(f, "Network error: {err}")
                }
            },
            DownloadError::IoError(err) => {
                match err.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        write!(f, "Permission denied: Unable to write to the specified location. Check file permissions and try running with appropriate privileges.")
                    },
                    std::io::ErrorKind::NotFound => {
                        write!(f, "Path not found: The specified directory does not exist and could not be created.")
                    },
                    std::io::ErrorKind::AlreadyExists => {
                        write!(f, "File conflict: Unable to create file because it already exists in an unexpected way.")
                    },
                    std::io::ErrorKind::InvalidInput => {
                        write!(f, "Invalid file path: The specified path contains invalid characters or is too long.")
                    },
                    std::io::ErrorKind::StorageFull => {
                        write!(f, "Storage full: Not enough disk space to save the file.")
                    },
                    _ => write!(f, "File system error: {err}"),
                }
            },
            DownloadError::InvalidUrl(msg) => write!(f, "Invalid URL: {msg}. Please check the URL format and try again."),
            DownloadError::ParseError(msg) => write!(f, "Parse error: {msg}. The content may be corrupted or in an unexpected format."),
            DownloadError::HttpError { status, url } => {
                match *status {
                    404 => write!(f, "File not found (404): The file at {url} does not exist on the server."),
                    403 => write!(f, "Access forbidden (403): You don't have permission to access {url}."),
                    401 => write!(f, "Unauthorized (401): Authentication required to access {url}."),
                    500..=599 => write!(f, "Server error ({status}): The server encountered an error while processing {url}."),
                    _ => write!(f, "HTTP error ({status}): Request to {url} failed."),
                }
            },
            DownloadError::FileNotFound(url) => write!(f, "File not found: {url} is not available on the server."),
            DownloadError::Timeout(url) => write!(f, "Timeout: Request to {url} took too long. The server may be overloaded."),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadError::NetworkError(err) => Some(err),
            DownloadError::IoError(err) => Some(err),
            DownloadError::InvalidUrl(_) => None,
            DownloadError::ParseError(_) => None,
            DownloadError::HttpError { .. } => None,
            DownloadError::FileNotFound(_) => None,
            DownloadError::Timeout(_) => None,
        }
    }
}

// Automatic conversions from common error types
impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::NetworkError(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::IoError(err)
    }
}

impl From<url::ParseError> for DownloadError {
    fn from(err: url::ParseError) -> Self {
        DownloadError::InvalidUrl(err.to_string())
    }
}

/// Result type for tracking download operations
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Successfully downloaded files with their local paths
    pub successful: Vec<(String, String)>, // (URL, local_path)
    /// Failed downloads with error messages and error types
    pub failed: Vec<(String, String, String)>, // (URL, error_message, error_type)
    /// Total number of files processed
    pub total_files: usize,
    /// Total bytes downloaded
    pub total_bytes: u64,
    /// Start time for duration calculation
    pub start_time: std::time::Instant,
}

impl DownloadResult {
    /// Create a new empty DownloadResult
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            total_files: 0,
            total_bytes: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Add a successful download
    pub fn add_success(&mut self, url: String, local_path: String, bytes: u64) {
        self.successful.push((url, local_path));
        self.total_files += 1;
        self.total_bytes += bytes;
    }

    /// Add a failed download
    pub fn add_failure(&mut self, url: String, error: String) {
        let error_type = self.categorize_error(&error);
        self.failed.push((url, error, error_type));
        self.total_files += 1;
    }

    /// Get the number of successful downloads
    pub fn success_count(&self) -> usize {
        self.successful.len()
    }

    /// Get the number of failed downloads
    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }

    /// Check if all downloads were successful
    pub fn all_successful(&self) -> bool {
        self.failed.is_empty() && !self.successful.is_empty()
    }

    /// Get the total duration of the download process
    pub fn duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Format bytes in a human-readable way
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// Categorize error for better reporting
    fn categorize_error(&self, error: &str) -> String {
        if error.contains("404") || error.contains("not found") || error.contains("Not Found") {
            "not_found".to_string()
        } else if error.contains("timeout") || error.contains("Timeout") {
            "timeout".to_string()
        } else if error.contains("Permission denied") || error.contains("permission") {
            "permission".to_string()
        } else if error.contains("Network") || error.contains("Connection") {
            "network".to_string()
        } else if error.contains("403") || error.contains("Forbidden") {
            "forbidden".to_string()
        } else if error.contains("500") || error.contains("502") || error.contains("503") {
            "server_error".to_string()
        } else {
            "other".to_string()
        }
    }

    /// Get error statistics by category
    pub fn error_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for (_, _, error_type) in &self.failed {
            *stats.entry(error_type.clone()).or_insert(0) += 1;
        }
        stats
    }
}

impl Default for DownloadResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DownloadResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let duration = self.duration();
        let duration_str = if duration.as_secs() > 60 {
            format!("{}m {:.1}s", duration.as_secs() / 60, (duration.as_secs() % 60) as f64 + duration.subsec_millis() as f64 / 1000.0)
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        };

        writeln!(f, "📊 Download Summary:")?;
        writeln!(f, "   ⏱️  Total time: {duration_str}")?;
        writeln!(f, "   📁 Total files processed: {}", self.total_files)?;
        writeln!(f, "   ✅ Successful downloads: {}", self.success_count())?;
        writeln!(f, "   ❌ Failed downloads: {}", self.failure_count())?;
        writeln!(f, "   💾 Total data downloaded: {}", Self::format_bytes(self.total_bytes))?;
        
        if self.total_files > 0 {
            let success_rate = (self.success_count() as f64 / self.total_files as f64) * 100.0;
            writeln!(f, "   📈 Success rate: {success_rate:.1}%")?;
            
            if duration.as_secs() > 0 && self.total_bytes > 0 {
                let speed = self.total_bytes as f64 / duration.as_secs_f64();
                writeln!(f, "   🚀 Average speed: {}/s", Self::format_bytes(speed as u64))?;
            }
        }
        
        if !self.successful.is_empty() {
            writeln!(f, "\n✅ Successfully downloaded files:")?;
            for (url, local_path) in &self.successful {
                writeln!(f, "   • {url}")?;
                writeln!(f, "     → {local_path}")?;
            }
        }
        
        if !self.failed.is_empty() {
            writeln!(f, "\n❌ Failed downloads:")?;
            
            // Group errors by type for better reporting
            let error_stats = self.error_stats();
            if error_stats.len() > 1 {
                writeln!(f, "\n📊 Error breakdown:")?;
                for (error_type, count) in &error_stats {
                    let description = match error_type.as_str() {
                        "not_found" => "Files not found (404)",
                        "timeout" => "Network timeouts",
                        "permission" => "Permission denied",
                        "network" => "Network/connection errors",
                        "forbidden" => "Access forbidden (403)",
                        "server_error" => "Server errors (5xx)",
                        _ => "Other errors",
                    };
                    writeln!(f, "   • {description}: {count} file(s)")?;
                }
                writeln!(f)?;
            }
            
            for (url, error, _) in &self.failed {
                writeln!(f, "   • {url}")?;
                writeln!(f, "     ❌ {error}")?;
            }
            
            writeln!(f, "\n💡 Troubleshooting recommendations:")?;
            
            let stats = self.error_stats();
            if stats.contains_key("network") || stats.contains_key("timeout") {
                writeln!(f, "   🌐 Network issues detected:")?;
                writeln!(f, "      • Check your internet connection stability")?;
                writeln!(f, "      • Verify the server is accessible from your location")?;
                writeln!(f, "      • Try again later if the server is overloaded")?;
            }
            
            if stats.contains_key("not_found") {
                writeln!(f, "   📄 Missing files detected:")?;
                writeln!(f, "      • Some files may have been moved or deleted")?;
                writeln!(f, "      • Check if the llms.txt file is up to date")?;
                writeln!(f, "      • Contact the website maintainer if many files are missing")?;
            }
            
            if stats.contains_key("permission") {
                writeln!(f, "   🔒 Permission issues detected:")?;
                writeln!(f, "      • Check write permissions for the download directory")?;
                writeln!(f, "      • Try running with elevated privileges if necessary")?;
                writeln!(f, "      • Ensure sufficient disk space is available")?;
            }
            
            if stats.contains_key("forbidden") {
                writeln!(f, "   🚫 Access restrictions detected:")?;
                writeln!(f, "      • Some files may require authentication")?;
                writeln!(f, "      • The server may be blocking automated requests")?;
                writeln!(f, "      • Try accessing the files manually in a browser")?;
            }
            
            if stats.contains_key("server_error") {
                writeln!(f, "   🔧 Server issues detected:")?;
                writeln!(f, "      • The server is experiencing technical difficulties")?;
                writeln!(f, "      • Try again later when the server is stable")?;
                writeln!(f, "      • Contact the website administrator if issues persist")?;
            }
        }
        
        Ok(())
    }
}