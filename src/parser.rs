use crate::error::DownloadError;
use url::Url;

/// Parse llms.txt content and extract file paths
/// 
/// This function processes the content of an llms.txt file and extracts
/// all file paths, converting relative paths to absolute URLs using the base URL.
/// Supports both plain file paths and markdown-style links.
/// 
/// # Arguments
/// * `content` - The raw content of the llms.txt file
/// * `base_url` - The base URL to resolve relative paths against
/// 
/// # Returns
/// * `Result<Vec<String>, DownloadError>` - Vector of absolute URLs or error
pub fn parse_llms_txt(content: &str, base_url: &str) -> Result<Vec<String>, DownloadError> {
    let mut file_urls = Vec::new();
    
    // Validate base URL
    let base = Url::parse(base_url)
        .map_err(|e| DownloadError::InvalidUrl(format!("Invalid base URL '{base_url}': {e}")))?;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Skip empty lines and comments (lines starting with #)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        // Extract file path from the line (handle markdown links and plain paths)
        if let Some(file_path) = extract_file_path(trimmed) {
            // Resolve the URL (convert relative to absolute if needed)
            let resolved_url = resolve_url(&file_path, &base)?;
            file_urls.push(resolved_url);
        }
    }
    
    Ok(file_urls)
}

/// Extract file path from a line, handling various formats
/// 
/// Supports:
/// - Plain file paths: `docs/api.md`
/// - Markdown links: `- [Title](/docs/api.md)`
/// - Markdown links with descriptions: `- [Title](/docs/api.md): Description`
/// 
/// # Arguments
/// * `line` - The line to extract the file path from
/// 
/// # Returns
/// * `Option<String>` - The extracted file path, or None if no valid path found
fn extract_file_path(line: &str) -> Option<String> {
    let trimmed = line.trim();
    
    // Handle markdown-style links: - [Title](/path.md) or - [Title](/path.md): Description
    if let Some(start) = trimmed.find("](") {
        if let Some(end) = trimmed[start + 2..].find(')') {
            let path = &trimmed[start + 2..start + 2 + end];
            // Only return paths that look like file paths (contain . or end with .md)
            if path.contains('.') || path.ends_with(".md") {
                return Some(path.to_string());
            }
        }
    }
    
    // Handle plain file paths (must contain a dot to be considered a file)
    if trimmed.contains('.') && !trimmed.starts_with('-') {
        return Some(trimmed.to_string());
    }
    
    None
}

/// Convert relative paths to absolute URLs using the base URL
/// 
/// This function takes a file path (which can be relative or absolute)
/// and converts it to an absolute URL using the provided base URL.
/// 
/// # Arguments
/// * `path` - The file path (relative or absolute URL)
/// * `base` - The base URL to resolve relative paths against
/// 
/// # Returns
/// * `Result<String, DownloadError>` - Absolute URL or error
pub fn resolve_url(path: &str, base: &Url) -> Result<String, DownloadError> {
    // If the path is already an absolute URL, use it as-is
    if let Ok(absolute_url) = Url::parse(path) {
        return Ok(absolute_url.to_string());
    }
    
    // Otherwise, resolve it as a relative path against the base URL
    let resolved = base.join(path)
        .map_err(|e| DownloadError::ParseError(format!("Failed to resolve URL '{path}' against base '{base}': {e}")))?;
    
    Ok(resolved.to_string())
}

