//! Disposable HTTP server for rendering components
//!
//! This server is NOT a daemon. It:
//! - Starts on a random port
//! - Serves one HTML document
//! - Shuts down after capture

use crate::{Result, VisionError};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tiny_http::{Response, Server};

/// Disposable server that serves a single HTML document
pub struct RenderServer {
    server: Server,
    port: u16,
    html_content: String,
}

impl RenderServer {
    /// Create a new server on a random available port
    pub fn new(html_content: String) -> Result<Self> {
        // Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| VisionError::Io(e))?;
        let port = listener
            .local_addr()
            .map_err(|e| VisionError::Io(e))?
            .port();

        // Create tiny_http server from listener
        let server = Server::from_listener(listener, None).map_err(|e| {
            VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(Self {
            server,
            port,
            html_content,
        })
    }

    /// Get the URL for accessing this server
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Start serving (blocks until stopped or timeout)
    pub fn serve_once(&self, timeout: Duration) -> Result<()> {
        // Set timeout on server
        let server = &self.server;

        // Accept one request
        if let Ok(Some(request)) = server.recv_timeout(timeout) {
            let response = Response::from_string(&self.html_content).with_header(
                tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=UTF-8"[..],
                )
                .unwrap(),
            );

            request.respond(response).map_err(|e| {
                VisionError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;
        }

        Ok(())
    }
}

/// Start a disposable server and return its URL
///
/// The server runs in a background thread and will accept one request.
pub fn start_disposable_server(html: String) -> Result<(String, thread::JoinHandle<()>)> {
    let server = RenderServer::new(html)?;
    let url = server.url();

    let handle = thread::spawn(move || {
        let _ = server.serve_once(Duration::from_secs(30));
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(100));

    Ok((url, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_url() {
        let html = "<html><body>Test</body></html>".to_string();
        let server = RenderServer::new(html).unwrap();

        let url = server.url();
        assert!(url.starts_with("http://127.0.0.1:"));
    }

    #[test]
    fn test_disposable_server() {
        let html = "<html><body>Test</body></html>".to_string();
        let (url, handle) = start_disposable_server(html).unwrap();

        assert!(url.starts_with("http://127.0.0.1:"));

        // Server should be accessible
        let response = reqwest::blocking::get(&url);
        // Note: This test requires reqwest to be added as dev dependency

        handle.join().unwrap();
    }
}
