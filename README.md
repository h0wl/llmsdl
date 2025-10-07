# llmsdl - LLMs.txt Downloader

A CLI tool written in Rust for downloading documentation files from websites that implement the [llms.txt standard](https://llmstxt.org/). This tool automatically discovers and downloads all documentation files listed in a website's `llms.txt` file, preserving the original directory structure locally.

## Features

- **Automatic Discovery**: Finds and parses `llms.txt` files from any website
- **Concurrent Downloads**: Downloads multiple files simultaneously for speed
- **Structure Preservation**: Maintains original directory structure locally
- **Multiple Formats**: Supports both plain file paths and markdown-style links in llms.txt

## Installation

### From Source

```bash
git clone https://github.com/h0wl/llmsdl
cd llmsdl
cargo build --release
```

The binary will be available at `target/release/llmsdl`.

### Using Cargo

```bash
cargo install --path .
```

## Usage

### Basic Usage

```bash
llmsdl https://docs.example.com
```

This will:

1. Look for `llms.txt` at `https://docs.example.com/llms.txt`
2. Parse the file to extract documentation URLs
3. Download all files to `downloads/docs.example.com/`
4. Preserve the original directory structure

### Examples

```bash
# Download from a documentation site
llmsdl https://docs.example.com

# Download from localhost (useful for development)
llmsdl http://localhost:3000

# Download from a site with custom port
llmsdl https://docs.example.com:8080
```

## Output Structure

Files are downloaded to a `downloads/` directory in your current working directory, organized by domain:

```
downloads/
└── docs.example.com/
    ├── README.md
    ├── api/
    │   ├── authentication.md
    │   └── endpoints.md
    └── guides/
        └── getting-started.md
```

## Supported llms.txt Formats

The tool supports various formats in `llms.txt` files:

### Plain File Paths

```
README.md
docs/api.md
guides/getting-started.md
```

### Markdown Links

```
- [About Us](/company/about.md)
- [API Documentation](/api/docs.md): Complete API reference
- [Getting Started](/guides/start.md)
```

### Mixed Content

```
# Documentation Files
README.md
- [API Reference](/api/reference.md)
https://external-site.com/shared-doc.md
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running with Debug Output

```bash
RUST_LOG=debug cargo run -- https://docs.example.com
```

## Project Structure

```
src/
├── main.rs           # CLI interface and main orchestration
├── error.rs          # Error types and handling
├── http_client.rs    # HTTP client with retry logic
├── parser.rs         # llms.txt parsing logic
└── file_manager.rs   # File system operations
```

## Dependencies

- **reqwest**: HTTP client for downloading files
- **tokio**: Async runtime for concurrent operations
- **clap**: Command-line argument parsing
- **url**: URL parsing and validation
- **indicatif**: Progress bars and spinners
- **futures**: Async utilities for concurrent downloads
- **anyhow**: Error handling utilities

## License

This project is licensed under the terms of the MIT license.
