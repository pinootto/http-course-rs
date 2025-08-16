use std::io::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

async fn read_response(tcp_stream: &mut TcpStream) -> Result<()> {
    // read response
    let mut response = String::new();
    let _ = tcp_stream.read_to_string(&mut response).await?;
    println!("response:");
    println!("{response}");
    println!();

    let mut line_iter = response.lines();
    // first line
    println!("first line:");
    let first_line = line_iter.next().unwrap();
    let mut tokens = first_line.split_whitespace();
    if let Some(protocol) = tokens.next() {
        println!("protocol = {}", protocol);
    } else {
        panic!();
    }
    if let Some(status) = tokens.next() {
        println!("status = {}", status)
    } else {
        panic!();
    }
    println!();

    // headers
    println!("headers:");
    loop {
        let line = line_iter.next();
        if let Some(header) = line {
            if !header.is_empty() {
                println!("header = {}", header);
            } else {
                break;
            }
        } else {
            panic!();
        }
    }
    println!();

    // body
    println!("body:");
    // while let Some(line) = line_iter.next() {
    //     println!("{}", line);
    // }
    for line in line_iter {
        println!("{}", line);
    }
    println!("--------------------------------");
    Ok(())
}

const ADDRESS_PORT: &str = "localhost:9000";

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the server
    let mut tcp_stream = TcpStream::connect(ADDRESS_PORT).await?;

    // send GET request
    tcp_stream
        .write_all(b"GET /headers HTTP/1.0\r\n\r\n")
        .await?;

    read_response(&mut tcp_stream).await?;

    // for each request we need to connect to the server again
    let mut tcp_stream = TcpStream::connect(ADDRESS_PORT).await?;

    // send GET request
    tcp_stream
        .write_all(b"GET /headers HTTP/1.0\r\n\r\n")
        .await?;

    read_response(&mut tcp_stream).await?;

    // for each request we need to connect to the server again
    let mut tcp_stream = TcpStream::connect(ADDRESS_PORT).await?;

    // send POST request
    tcp_stream.write_all(b"POST /echo HTTP/1.0\r\n").await?;
    // headers
    let request_body = "name=pippo&age=3";
    let header_1 = format!("content-length: {}\r\n", request_body.len());
    tcp_stream.write_all(header_1.as_bytes()).await?;
    tcp_stream
        .write_all(b"content-type: application/x-www-form-urlencoded\r\n")
        .await?;
    // empty line
    tcp_stream.write_all(b"\r\n").await?;
    // body
    tcp_stream.write_all(request_body.as_bytes()).await?;

    read_response(&mut tcp_stream).await?;

    Ok(())
}
