use std::io::Result;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8091").await?;
    println!("server listening on {}", listener.local_addr().unwrap());
    loop {
        let (mut tcp_stream, _client_socket_address) = listener.accept().await?;
        // dbg!(&tcp_stream);
        // dbg!(client_socket_address);
        let (mut reader, mut writer) = tcp_stream.split();
        let mut buffer: [u8; 200] = [0; 200];
        let _n = reader.read(&mut buffer).await?;
        // dbg!(buffer);
        let mut lines = buffer.lines();
        if let Some(first_line) = lines.next_line().await? {
            println!("{first_line}");
            let mut words = first_line.split_whitespace();
            if let Some(method) = words.next()
                && method == "GET"
            {
                println!("received a GET request");
            } else {
                println!("invalid request");
                continue;
            };
            if let Some(path) = words.next() {
                println!("path = {}", path);
                let response_html = format!("<html><title>Welcome</title><body><h1>HTTP 0.9</h1><p>you requested {}</p></body></html>", path);
                writer.write_all(response_html.as_bytes()).await?;
            } else {
                println!("no path");
                continue;
            }
        }
    }
}
