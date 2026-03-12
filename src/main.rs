use std::net::TcpListener;
use std::io::Write;

fn main() -> std::io::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // TODO: Uncomment the code below to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
