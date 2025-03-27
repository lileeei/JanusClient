# Janus Client

A unified browser debugging protocol client in Rust. Currently supports Chrome DevTools Protocol (CDP) with plans for Firefox DevTools Protocol (FDP) support.

## Features

- Unified interface for browser automation and debugging
- Async/await based API
- Strong type safety with Rust's type system
- Modular design with trait-based abstractions
- Comprehensive error handling
- Built-in logging and tracing support

## Supported Browsers

- Chrome/Chromium (via Chrome DevTools Protocol)
- Firefox (coming soon)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
janus_client = "0.1.0"
```

## Quick Start

```rust
use janus_client::core::BrowserDebugger;
use janus_client::implementations::chrome::ChromeDebugger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new Chrome debugger instance
    let mut debugger = ChromeDebugger::new();

    // Connect to Chrome DevTools
    debugger.connect("ws://localhost:9222/devtools/browser").await?;

    // Create a new page and navigate to a URL
    let mut page = debugger.create_page(Some("https://www.rust-lang.org")).await?;
    
    // Take a screenshot
    let screenshot = page.take_screenshot("png").await?;
    std::fs::write("screenshot.png", screenshot)?;

    // Clean up
    debugger.disconnect().await?;
    
    Ok(())
}
```

## Usage

### Starting Chrome for Debugging

Start Chrome with remote debugging enabled:

```bash
# Linux/macOS
google-chrome --remote-debugging-port=9222

# Windows
"C:\Program Files\Google\Chrome\Application\chrome.exe" --remote-debugging-port=9222
```

### Core Concepts

The library is built around several key abstractions:

- `BrowserDebugger`: The main entry point for browser automation
- `Page`: Represents a browser tab/page
- `Dom`: DOM manipulation interface
- `Network`: Network monitoring interface

### Examples

See the `examples` directory for more detailed examples:

- `chrome_example.rs`: Demonstrates Chrome automation
- `firefox_example.rs`: Firefox automation example (coming soon)

## Architecture

The library follows a layered architecture:

1. **Core Layer**: Defines abstract traits and interfaces
2. **Adapter Layer**: Protocol-specific adapters
3. **Implementation Layer**: Concrete implementations for different browsers

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
# Run Chrome example
cargo run --example chrome_example

# Run Firefox example (coming soon)
cargo run --example firefox_example
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
