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
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // connect to the server
    let mut tcp_stream = TcpStream::connect("127.0.0.1:9000").await?;

    // send request
    tcp_stream
        .write_all(b"GET /headers HTTP/1.0\r\n\r\n")
        .await?;

    read_response(&mut tcp_stream).await?;

    // connect to the server
    let mut tcp_stream = TcpStream::connect("127.0.0.1:9000").await?;

    // send request
    tcp_stream
        .write_all(b"GET /headers HTTP/1.0\r\n\r\n")
        .await?;

    read_response(&mut tcp_stream).await?;

    // // read response
    // let mut response = String::new();
    // let _ = tcp_stream.read_to_string(&mut response).await?;
    // println!("response:");
    // println!("{response}");
    // println!();
    //
    // let mut line_iter = response.lines();
    // // first line
    // println!("first line:");
    // let first_line = line_iter.next().unwrap();
    // let mut tokens = first_line.split_whitespace();
    // if let Some(protocol) = tokens.next() {
    //     println!("protocol = {}", protocol);
    // } else {
    //     panic!();
    // }
    // if let Some(status) = tokens.next() {
    //     println!("status = {}", status)
    // } else {
    //     panic!();
    // }
    // println!();
    //
    // // headers
    // println!("headers:");
    // loop {
    //     let line = line_iter.next();
    //     if let Some(header) = line {
    //         if !header.is_empty() {
    //             println!("header = {}", header);
    //         } else {
    //             break;
    //         }
    //     } else {
    //         panic!();
    //     }
    // }
    // println!();
    //
    // // body
    // println!("body:");
    // // while let Some(line) = line_iter.next() {
    // //     println!("{}", line);
    // // }
    // for line in line_iter {
    //     println!("{}", line);
    // }
    //
    Ok(())
}
