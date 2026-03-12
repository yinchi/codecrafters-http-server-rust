use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::vec;

const HOST: &str = "127.0.0.1:4221";
const NUM_THREADS: usize = 4;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(HOST).unwrap();
    let pool = threadpool::ThreadPool::new(NUM_THREADS);

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                // Spawn a new thread to handle the request
                pool.execute(move || {
                    if let Err(e) = handle_client(&mut _stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
#[allow(dead_code)]
struct Request {
    // The HTTP method (e.g., GET, POST)
    method: String,
    // The requested path (e.g., /index.html)
    path: String,
    // The HTTP version (e.g., HTTP/1.1)
    version: String,
    // The headers as a HashMap of (key, value) pairs
    headers: HashMap<String, String>,
    // The body of the request (if any)
    body: Option<String>,
}

fn handle_client(stream: &mut std::net::TcpStream) -> std::io::Result<()> {
    let request = Request::from_stream(stream)?;
    let (status, response) = handle_request(&request);
    println!("{} {} {}", request.method, request.path, status);
    stream.write_all(response.as_bytes())?;
    Ok(())
}

impl Request {
    fn from_stream(stream: &mut std::net::TcpStream) -> std::io::Result<Self> {
        let mut reader = BufReader::new(stream);

        // Parse the request line
        // The request line should be in the format: METHOD PATH VERSION
        let mut line_buf = String::new();

        reader.read_line(&mut line_buf)?;

        let mut parts = line_buf.split_whitespace();
        let method = parts.next().ok_or(IoError::new(
            IoErrorKind::InvalidData,
            "Missing HTTP method",
        ))?;
        let path = parts.next().ok_or(IoError::new(
            IoErrorKind::InvalidData,
            "Missing request path",
        ))?;
        let version = parts.next().ok_or(IoError::new(
            IoErrorKind::InvalidData,
            "Missing HTTP version",
        ))?;

        // Parse headers
        // Each header should be in the format: Key: Value
        let mut headers = HashMap::new();
        loop {
            let mut line_buf = String::new();
            let bytes_read = reader.read_line(&mut line_buf)?;
            if bytes_read == 0 || line_buf.trim().is_empty() {
                break; // End of headers
            }
            if let Some((key, value)) = line_buf.split_once(": ") {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            } else {
                return Err(IoError::new(
                    IoErrorKind::InvalidData,
                    "Invalid header format",
                ));
            }
        }

        // After the headers, read the request body if Content-Length header is present
        let mut body: Option<String> = None;
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length: usize = content_length.parse().map_err(|_| {
                IoError::new(IoErrorKind::InvalidData, "Invalid Content-Length value")
            })?;
            let mut body_buf = vec![0; content_length];
            reader.read_exact(&mut body_buf)?;
            body = Some(String::from_utf8_lossy(&body_buf).to_string());
        }

        Ok(Request {
            method: method.to_string(),
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body,
        })
    }
}

fn handle_request(request: &Request) -> (u16, String) {
    // Return HTTP 200 on the root path, and 404 on any other path
    // println!("Received request: {:?}", request);
    match request.path.as_str() {
        "/" => handle_root(),
        s if s.starts_with("/echo/") => handle_echo(request),
        s if s.starts_with("/user-agent") => handle_user_agent(request),
        _ => handle_404(),
    }
}

fn handle_root() -> (u16, String) {
    (
        200,
        ["HTTP/1.1 200 OK", "Content-Length: 0", "", ""].join("\r\n"),
    )
}

fn handle_echo(request: &Request) -> (u16, String) {
    let path = request.path.strip_prefix("/echo/").unwrap_or("");
    (
        200,
        [
            "HTTP/1.1 200 OK",
            "Content-Type: text/plain",
            &format!("Content-Length: {}", path.len()),
            "",
            path,
        ]
        .join("\r\n"),
    )
}

fn handle_user_agent(request: &Request) -> (u16, String) {
    let user_agent = get_header_else(request, "User-Agent", "Unknown");
    (
        200,
        [
            "HTTP/1.1 200 OK",
            "Content-Type: text/plain",
            &format!("Content-Length: {}", user_agent.len()),
            "",
            &user_agent,
        ]
        .join("\r\n"),
    )
}

fn handle_404() -> (u16, String) {
    (
        404,
        ["HTTP/1.1 404 Not Found", "Content-Length: 0", "", ""].join("\r\n"),
    )
}

fn get_header(request: &Request, header_name: &str) -> Option<String> {
    request.headers.get(header_name).cloned()
}

fn get_header_else(request: &Request, header_name: &str, default: &str) -> String {
    get_header(request, header_name).unwrap_or_else(|| default.to_string())
}
