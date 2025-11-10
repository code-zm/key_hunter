use crate::core::error::{KeyHunterError, Result};
use curl::easy::{Easy2, Handler, WriteError};
use std::time::Duration;

/// Collector for response data
struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

/// HTTP client using libcurl
pub struct HttpClient {
    timeout: Duration,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Perform a GET request
    pub fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse> {
        let mut easy = Easy2::new(Collector(Vec::new()));

        easy.url(url)?;
        easy.timeout(self.timeout)?;
        easy.follow_location(true)?;
        easy.max_redirections(5)?;
        easy.ssl_verify_peer(true)?;
        easy.ssl_verify_host(true)?;

        // Set headers
        let mut list = curl::easy::List::new();
        for (key, value) in headers {
            list.append(&format!("{}: {}", key, value))?;
        }
        easy.http_headers(list)?;

        // Perform the request
        easy.perform()?;

        let response_code = easy.response_code()?;
        let body = easy.get_ref().0.clone();

        Ok(HttpResponse {
            status_code: response_code as u16,
            body,
        })
    }

    /// Perform a POST request
    pub fn post(&self, url: &str, headers: &[(&str, &str)], body: &str) -> Result<HttpResponse> {
        let mut easy = Easy2::new(Collector(Vec::new()));

        easy.url(url)?;
        easy.timeout(self.timeout)?;
        easy.post(true)?;
        easy.post_fields_copy(body.as_bytes())?;
        easy.follow_location(true)?;
        easy.max_redirections(5)?;
        easy.ssl_verify_peer(true)?;
        easy.ssl_verify_host(true)?;

        // Set headers
        let mut list = curl::easy::List::new();
        for (key, value) in headers {
            list.append(&format!("{}: {}", key, value))?;
        }
        easy.http_headers(list)?;

        // Perform the request
        easy.perform()?;

        let response_code = easy.response_code()?;
        let body = easy.get_ref().0.clone();

        Ok(HttpResponse {
            status_code: response_code as u16,
            body,
        })
    }

    /// Perform a GET request and parse JSON
    pub fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<(u16, T)> {
        let response = self.get(url, headers)?;
        let parsed = serde_json::from_slice(&response.body)?;
        Ok((response.status_code, parsed))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| KeyHunterError::Unknown(format!("Invalid UTF-8: {}", e)))
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(Into::into)
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    pub fn is_rate_limited(&self) -> bool {
        self.status_code == 403 || self.status_code == 429
    }

    pub fn is_not_found(&self) -> bool {
        self.status_code == 404
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        assert_eq!(client.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_http_client_custom_timeout() {
        let client = HttpClient::with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }
}
