//! Downloading over HTTP(S) for repository rules (buildfiji-mum.8.3).
//!
//! What Bazel 9.2.0 does, read off probes: a `4xx` is final (`GET returned 404
//! Not Found`), a `5xx` or a failed connection is tried again up to eight
//! times in all (the last failure is the answer), `headers` are sent as given,
//! and `auth` for a URL is `basic` (a `login` and `password`) or `pattern` (a
//! `pattern` in which `<password>` is replaced, sent as the `Authorization`
//! header).

use base64_wrap::encode;
use fjfj_starlark::{Downloader, HttpRequest};
use reqwest::blocking::Client;
use std::time::Duration;

/// `base64` for the one header that needs it, without another dependency.
mod base64_wrap {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(input: &[u8]) -> String {
        let mut out = String::new();
        for chunk in input.chunks(3) {
            let b = [
                chunk[0],
                *chunk.get(1).unwrap_or(&0),
                *chunk.get(2).unwrap_or(&0),
            ];
            let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
            for i in 0..4 {
                if i <= chunk.len() {
                    out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }
}

/// A [`Downloader`] over `reqwest`.
pub struct HttpDownloader {
    client: Client,
    attempts: u32,
    backoff: Duration,
}

impl HttpDownloader {
    /// A downloader that tries each URL up to `attempts` times, waiting
    /// `backoff` times the attempt number between tries.
    pub fn new(attempts: u32, backoff: Duration) -> Result<HttpDownloader, String> {
        let client = Client::builder()
            .user_agent(concat!("fjfj/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(HttpDownloader {
            client,
            attempts: attempts.max(1),
            backoff,
        })
    }

    /// Bazel's own: eight attempts with a short wait.
    pub fn standard() -> Result<HttpDownloader, String> {
        HttpDownloader::new(8, Duration::from_millis(250))
    }
}

fn authorization(auth: &[(String, String)]) -> Option<String> {
    let field = |name: &str| {
        auth.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    };
    match field("type")? {
        "basic" => Some(format!(
            "Basic {}",
            encode(format!("{}:{}", field("login")?, field("password")?).as_bytes())
        )),
        "pattern" => {
            Some(field("pattern")?.replace("<password>", field("password").unwrap_or_default()))
        }
        _ => None,
    }
}

impl Downloader for HttpDownloader {
    fn get(&self, request: &HttpRequest) -> Result<Vec<u8>, String> {
        let mut last = String::new();
        for attempt in 0..self.attempts {
            if attempt > 0 {
                std::thread::sleep(self.backoff * attempt);
            }
            let mut builder = self.client.get(&request.url);
            for (name, values) in &request.headers {
                for value in values {
                    builder = builder.header(name.as_str(), value.as_str());
                }
            }
            if let Some(value) = authorization(&request.auth) {
                builder = builder.header("Authorization", value);
            }
            match builder.send() {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return response
                            .bytes()
                            .map(|b| b.to_vec())
                            .map_err(|e| e.to_string());
                    }
                    let message = format!(
                        "GET returned {} {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    )
                    .trim_end()
                    .to_owned();
                    if status.is_client_error() {
                        return Err(message);
                    }
                    last = message;
                }
                Err(e) => last = e.to_string(),
            }
        }
        Err(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A server that answers each request with `respond(path, headers)`.
    fn serve(
        respond: impl Fn(&str, &str) -> (u16, &'static str, Vec<u8>) + Send + Sync + 'static,
    ) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let count = Arc::new(AtomicUsize::new(0));
        let seen = count.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };
                let mut buffer = [0u8; 4096];
                let n = stream.read(&mut buffer).unwrap_or(0);
                let text = String::from_utf8_lossy(&buffer[..n]).into_owned();
                seen.fetch_add(1, Ordering::SeqCst);
                let path = text.split_whitespace().nth(1).unwrap_or("/").to_owned();
                let (status, reason, body) = respond(&path, &text);
                let head = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        (url, count)
    }

    fn request(url: String) -> HttpRequest {
        HttpRequest {
            url,
            headers: Vec::new(),
            auth: Vec::new(),
        }
    }

    #[test]
    fn a_successful_get_returns_the_body_and_sends_headers_and_basic_auth() {
        let (url, _) = serve(|_, text| {
            // Header names go out in lower case.
            let lower = text.to_lowercase();
            let ok = lower.contains("x-a: b") && lower.contains("authorization: basic dtpw");
            (200, "OK", if ok { b"yes".to_vec() } else { b"no".to_vec() })
        });
        let mut r = request(format!("{url}/f"));
        r.headers = vec![("X-A".to_owned(), vec!["b".to_owned()])];
        r.auth = vec![
            ("type".to_owned(), "basic".to_owned()),
            ("login".to_owned(), "u".to_owned()),
            ("password".to_owned(), "p".to_owned()),
        ];
        let d = HttpDownloader::new(1, Duration::ZERO).unwrap();
        assert_eq!(d.get(&r), Ok(b"yes".to_vec()));
    }

    #[test]
    fn a_client_error_is_final_and_a_server_error_is_tried_again() {
        let (url, count) = serve(|path, _| match path {
            "/gone" => (404, "Not Found", Vec::new()),
            _ => (500, "Internal Server Error", Vec::new()),
        });
        let d = HttpDownloader::new(3, Duration::ZERO).unwrap();
        assert_eq!(
            d.get(&request(format!("{url}/gone"))),
            Err("GET returned 404 Not Found".to_owned())
        );
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert_eq!(
            d.get(&request(format!("{url}/broken"))),
            Err("GET returned 500 Internal Server Error".to_owned())
        );
        assert_eq!(count.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn a_pattern_auth_puts_the_password_in_the_header() {
        let auth = vec![
            ("type".to_owned(), "pattern".to_owned()),
            ("pattern".to_owned(), "Bearer <password>".to_owned()),
            ("password".to_owned(), "tok".to_owned()),
        ];
        assert_eq!(authorization(&auth).as_deref(), Some("Bearer tok"));
    }

    #[test]
    fn base64_matches_the_standard_examples() {
        assert_eq!(encode(b"u:p"), "dTpw");
        assert_eq!(encode(b"ab"), "YWI=");
        assert_eq!(encode(b"a"), "YQ==");
    }
}
