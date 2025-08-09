use std::io::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:8091").await?;
    tcp_stream.write_all(b"GET / HTTP/0.9").await?;
    let mut response = String::new();
    let _ = tcp_stream.read_to_string(&mut response).await?;
    println!("{response}");
    Ok(())
}
