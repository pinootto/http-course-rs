use std::string::ToString;
use std::sync::Arc;
use std::{collections::HashMap, io::Result};
use strum_macros::Display;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;
use tokio_rustls::{
    client::TlsStream,
    rustls::{pki_types::ServerName, ClientConfig, RootCertStore},
};
use url::Url;

#[derive(Debug)]
struct HttpConnection {
    tls_stream: TlsStream<TcpStream>,
}

impl HttpConnection {
    async fn new(host: String, port: u16) -> Result<Self> {
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let tls_connector = TlsConnector::from(Arc::new(config));
        let dnsname = ServerName::try_from(host.clone()).unwrap();
        let url = format!("{}:{}", host, port);

        // connect to the server
        let tcp_stream = TcpStream::connect(url.as_str()).await?;
        let tls_stream = tls_connector.connect(dnsname.clone(), tcp_stream).await?;
        Ok(Self { tls_stream })
    }

    async fn send(&mut self, request: HttpRequest) -> Result<HttpResponse> {
        // write the request
        // first line
        let request_line = format!("{} {} HTTP/1.1\r\n", request.method, request.relative_url());
        self.tls_stream.write_all(request_line.as_bytes()).await?;

        // headers
        for header in request.headers {
            let (key, value) = header;
            let header_line = format!("{}: {}\r\n", key, value);
            self.tls_stream.write_all(header_line.as_bytes()).await?;
        }
        let empty_line = format!("\r\n");
        self.tls_stream.write_all(empty_line.as_bytes()).await?;

        // body
        self.tls_stream.write_all(request.body.as_slice());

        self.tls_stream.flush().await?;

        // read the response
        let mut response = HttpResponse::default();
        // first line
        let mut line = String::new();
        let n_bytes = self.tls_stream.read_line(&mut line).await?;
        println!("read {} bytes in line: {}", n_bytes, line);
        println!();

        println!("first line:");
        let mut tokens = line.split_whitespace();
        if let Some(protocol) = tokens.next() {
            println!("protocol = {}", protocol);
        } else {
            panic!();
        }
        if let Some(status) = tokens.next() {
            println!("status = {}", status);
            response.status = status.parse().unwrap();
        } else {
            panic!();
        }
        println!();

        // headers
        println!("headers:");
        // let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            let _ = self.tls_stream.read_line(&mut line).await?;
            if !line.trim().is_empty() {
                let key_value = line.split_once(":").unwrap();
                response.headers.insert(
                    key_value.0.trim().to_string(),
                    key_value.1.trim().to_string(),
                );
            } else {
                break;
            }
        }
        for header in &response.headers {
            println!("{}: {}", header.0, header.1);
        }
        println!();

        // body
        println!("body:");
        let body_length = response
            .headers
            .get("Content-Length")
            .ok_or_else(|| std::io::Error::other("missing Content-Length header"))?;
        println!("Content-Length = {}", body_length);
        let size: usize = body_length.parse().unwrap();
        let mut body: Vec<u8> = Vec::new();
        for _ in 0..size {
            body.push(self.tls_stream.read_u8().await?);
        }
        println!("{}", String::from_utf8(body.clone()).unwrap());
        println!("--------------------------------");
        response.body = body;

        Ok(response)
    }
}

#[derive(Display, Debug)]
enum HttpMethod {
    #[strum(serialize = "GET")]
    Get,
    #[strum(serialize = "HEAD")]
    Head,
    #[strum(serialize = "POST")]
    Post,
    #[strum(serialize = "PUT")]
    Put,
    #[strum(serialize = "PATCH")]
    Patch,
    #[strum(serialize = "DELETE")]
    Delete,
    #[strum(serialize = "OPTIONS")]
    Options,
}

#[derive(Debug)]
struct HttpRequest {
    method: HttpMethod,
    uri: Url,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn relative_url(&self) -> String {
        let path = &self.uri.path();
        let query = &self.uri.query();
        match query {
            Some(query) => format!("{path}?{query}"),
            None => path.to_string(),
        }
    }

    fn get(uri: Url) -> Self {
        let mut headers = HashMap::new();
        headers.insert("host".to_string(), uri.host_str().unwrap().to_string());
        let body: Vec<u8> = Vec::new();
        Self {
            method: HttpMethod::Get,
            uri,
            headers,
            body,
        }
    }
}

#[derive(Default, Debug)]
struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
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
        if !line.trim().is_empty() {
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
    let body_length = headers
        .get("Content-Length")
        .ok_or_else(|| std::io::Error::other("missing Content-Length header"))?;
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
const PORT: u16 = 443;

#[tokio::main]
async fn main() -> Result<()> {
    // handle TLS and certificates
    // let mut root_cert_store = RootCertStore::empty();
    // root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    // let config = ClientConfig::builder()
    //     .with_root_certificates(root_cert_store)
    //     .with_no_client_auth();
    // let tls_connector = TlsConnector::from(Arc::new(config));
    // let dnsname = ServerName::try_from(DOMAIN).unwrap();
    // let url = format!("{}:{}", DOMAIN, PORT);

    // // connect to the server
    // let tcp_stream = TcpStream::connect(url.as_str()).await?;
    // let mut tls_stream = tls_connector.connect(dnsname.clone(), tcp_stream).await?;

    let mut http_connection = HttpConnection::new(DOMAIN.to_string(), PORT).await?;

    let request = HttpRequest::get(Url::parse("https://gioyingtec.com").unwrap());
    http_connection.send(request).await?;

    let request = HttpRequest::get(Url::parse("https://gioyingtec.com").unwrap());
    http_connection.send(request).await?;

    // let mut tls_stream = http_connection.tls_stream;
    // // send GET request
    // tls_stream.write_all(b"GET / HTTP/1.1\r\n").await?;
    // // tls_stream
    // //     .write_all(b"content-type: text/plain\r\n")
    // //     .await?;
    // tls_stream.write_all(b"host: gioyingtec.com\r\n").await?;
    // tls_stream.write_all(b"\r\n").await?;
    //
    // read_response(&mut tls_stream).await?;
    //
    // // we can reuse the connection for a new request
    //
    // // send GET request
    // tls_stream.write_all(b"GET / HTTP/1.1\r\n").await?;
    // tls_stream.write_all(b"host: gioyingtec.com\r\n").await?;
    // tls_stream.write_all(b"\r\n").await?;
    //
    // read_response(&mut tls_stream).await?;
    //
    // // we can reuse the connection for a new request
    //
    // // send POST request
    // tls_stream.write_all(b"POST /echo HTTP/1.1\r\n").await?;
    // // headers
    // let request_body = "name=pippo&age=3";
    // let header_1 = format!("content-length: {}\r\n", request_body.len());
    // tls_stream.write_all(header_1.as_bytes()).await?;
    // tls_stream
    //     .write_all(b"content-type: application/x-www-form-urlencoded\r\n")
    //     .await?;
    // // empty line
    // tls_stream.write_all(b"\r\n").await?;
    // // body
    // tls_stream.write_all(request_body.as_bytes()).await?;
    //
    // read_response(&mut tls_stream).await?;

    Ok(())
}
