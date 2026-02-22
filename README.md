# llmsdl - LLMs.txt Downloader

A CLI tool written in Rust for downloading documentation files from websites that implement the [llms.txt standard](https://llmstxt.org/). This tool automatically discovers and downloads all documentation files listed in a website's `llms.txt` file, preserving the original directory structure locally.

## Features

- **Automatic Discovery**: Finds and parses `llms.txt` files from any website
- **Direct llms.txt URL**: Pass a direct URL to an `llms.txt` file (e.g. `https://example.com/docs/llms.txt`) and the tool will use it and derive the base URL
- **Markdown-friendly URLs**: For doc links without a file extension (e.g. Elastic-style `/docs/get-started`), the tool appends `.md` so you get markdown instead of HTML
- **Concurrent Downloads**: Downloads multiple files simultaneously for speed
- **Structure Preservation**: Maintains original directory structure locally
- **Multiple Formats**: Supports both plain file paths and markdown-style links in llms.txt

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (rustc, cargo). Install with: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Build and install (recommended)

From the project directory:

```bash
cargo install --path .
```

This compiles the project in release mode and installs the `llmsdl` binary into Cargo’s bin directory (e.g. `~/.cargo/bin/llmsdl`). Ensure that directory is on your `PATH`.

### Build from source (binary only)

```bash
git clone https://github.com/h0wl/llmsdl
cd llmsdl
cargo build --release
```

The binary will be at `target/release/llmsdl`. Run it with `./target/release/llmsdl` or copy/link it to a directory on your `PATH`.

## Usage

### Basic Usage

```bash
llmsdl https://docs.example.com -o ./downloads
```

You can pass either a **base URL** or a **direct llms.txt URL**:

- **Base URL**: The tool looks for `llms.txt` at `{url}/llms.txt`.
- **Direct llms.txt URL**: Use the full URL to the file (e.g. `https://www.elastic.co/docs/llms.txt`). The tool fetches that file and derives the base URL for resolving links.

The tool will:

1. Fetch `llms.txt` (from the given URL or from `{base}/llms.txt`)
2. Parse it to extract documentation URLs (appending `.md` when a link has no file extension, so you get markdown instead of HTML where supported)
3. Download all files under the output directory, organized by domain
4. Preserve the original directory structure

### Examples

```bash
# Base URL: look for llms.txt at that origin
llmsdl https://docs.example.com -o ./downloads

# Direct llms.txt URL (e.g. Elastic docs)
llmsdl https://www.elastic.co/docs/llms.txt -o ./elastic-docs

# Localhost (development)
llmsdl http://localhost:3000 -o ./downloads

# Custom port
llmsdl https://docs.example.com:8080 -o ./downloads

# Limit concurrency (default is 5)
llmsdl https://docs.example.com -o ./downloads -t 3
```

## Output Structure

Files are saved under the directory you pass with `-o` (e.g. `./downloads`), organized by domain:

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

### Extensionless doc links

For links without a file extension (e.g. `https://example.com/docs/get-started` or `](/docs/get-started)`), the tool appends `.md` when fetching, so sites that serve markdown at `.../path.md` (e.g. [Elastic’s llms.txt](https://www.elastic.co/docs/llms.txt)) are downloaded as markdown instead of HTML.

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
