# tidepool 🦀🌊

A Simple Rust Written Web-Server after learning Rust

## Features

- Single-threaded → multithreaded with a custom thread pool
- Serves static files from a `public/` folder
- Graceful shutdown on Ctrl+C
- Proper HTTP/1.1 responses (200 OK, 404 Not Found, 500, etc.)
- Sleeps 5 seconds on `/sleep` to demonstrate thread pool behavior

## Usage

```bash
git clone https://github.com/VkianV/tidepool.git
cd tidepool
cargo run --release