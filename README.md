# tidepool 🦀🌊

A minimal Rust Written multithreaded Web-Server using threadpool

## Features

- Single-threaded → multithreaded with a custom thread pool
- Serves static files from a `public/` folder in its directory.
- Graceful shutdown on Ctrl+C
- Proper HTTP/1.1 responses (200 OK, 404 Not Found, 404, etc.)
- Sleeps 5 seconds on `/sleep` to demonstrate thread pool behavior

## Usage

```bash
git clone https://github.com/VkianV/tidepool.git
cd tidepool
cargo run --release
```
it is recommended to clone and compile the source code using cargo on your system, though the binary of this project is 
also available in the release section of this page.

don't forget to add your own public directory to the project with its html/css/js files.