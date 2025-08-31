use std::sync::Arc;
use std::usize;
use std::{collections::HashMap, io::Result};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;
use tokio_rustls::{
    client::TlsStream,
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
};

async fn read_line(tls_stream: &mut TlsStream<TcpStream>) -> Result<String> {
    let mut line = String::new();
    let n_bytes = tls_stream.read_line(&mut line).await?;
    println!("read {} bytes in line: {}", n_bytes, line);
    Ok(line)
}

async fn read_response(tls_stream: &mut TlsStream<TcpStream>) -> Result<()> {
    let mut line = String::new();
    let n_bytes = tls_stream.read_line(&mut line).await?;
    println!("read {} bytes in line: {}", n_bytes, line);
    println!();

    // first line
    println!("first line:");
    let mut tokens = line.split_whitespace();
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
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        let _ = tls_stream.read_line(&mut line).await?;
        // println!("read {} bytes in line: {}", n_bytes, line);
        if !line.trim().is_empty() {
            // println!("header = {}", line);
            let key_value = line.split_once(":").unwrap();
            headers.insert(
                key_value.0.trim().to_string(),
                key_value.1.trim().to_string(),
            );
        } else {
            break;
        }
    }
    for header in &headers {
        println!("{}: {}", header.0, header.1);
    }
    println!();

    // body
    println!("body:");
    //todo
    let body_length = headers.get("Content-Length").unwrap();
    println!("Content-Length = {}", body_length);
    let size: usize = body_length.parse().unwrap();
    let mut body: Vec<u8> = Vec::new();
    for _ in 0..size {
        body.push(tls_stream.read_u8().await?);
    }
    println!("{}", String::from_utf8(body).unwrap());
    println!("--------------------------------");
    Ok(())
}

const DOMAIN: &str = "gioyingtec.com";
const PORT: &str = "443";

#[tokio::main]
async fn main() -> Result<()> {
    // handle TLS and certificates
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let tls_connector = TlsConnector::from(Arc::new(config));
    let dnsname = ServerName::try_from(DOMAIN).unwrap();
    let url = format!("{}:{}", DOMAIN, PORT);
    // connect to the server
    let tcp_stream = TcpStream::connect(url.as_str()).await?;
    let mut tls_stream = tls_connector.connect(dnsname.clone(), tcp_stream).await?;

    // send GET request
    tls_stream.write_all(b"GET / HTTP/1.1\r\n").await?;
    // tls_stream
    //     .write_all(b"content-type: text/plain\r\n")
    //     .await?;
    tls_stream.write_all(b"host: gioyingtec.com\r\n").await?;
    tls_stream.write_all(b"\r\n").await?;

    read_response(&mut tls_stream).await?;

    // for each request we need to connect to the server again
    let tcp_stream = TcpStream::connect(url.as_str()).await?;
    let mut tls_stream = tls_connector.connect(dnsname.clone(), tcp_stream).await?;

    // send GET request
    tls_stream.write_all(b"GET /headers HTTP/1.0\r\n").await?;
    tls_stream.write_all(b"host: gioyingtec.com\r\n").await?;
    tls_stream.write_all(b"\r\n").await?;

    read_response(&mut tls_stream).await?;

    // for each request we need to connect to the server again
    let tcp_stream = TcpStream::connect(url.as_str()).await?;
    let mut tls_stream = tls_connector.connect(dnsname, tcp_stream).await?;

    // send POST request
    tls_stream.write_all(b"POST /echo HTTP/1.0\r\n").await?;
    // headers
    let request_body = "name=pippo&age=3";
    let header_1 = format!("content-length: {}\r\n", request_body.len());
    tls_stream.write_all(header_1.as_bytes()).await?;
    tls_stream
        .write_all(b"content-type: application/x-www-form-urlencoded\r\n")
        .await?;
    // empty line
    tls_stream.write_all(b"\r\n").await?;
    // body
    tls_stream.write_all(request_body.as_bytes()).await?;

    read_response(&mut tls_stream).await?;

    Ok(())
}
