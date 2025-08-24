use std::io::Result;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;
use tokio_rustls::{
    client::TlsStream,
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
};

async fn read_response(tls_stream: &mut TlsStream<TcpStream>) -> Result<()> {
    // read response
    let mut response = String::new();
    let _ = tls_stream.read_to_string(&mut response).await?;
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

// const ADDRESS_PORT: &str = "localhost:9000";
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
    tls_stream.write_all(b"GET / HTTP/1.0\r\n").await?;
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
