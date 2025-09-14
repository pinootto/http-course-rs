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
        let empty_line = "\r\n".to_string();
        self.tls_stream.write_all(empty_line.as_bytes()).await?;

        // body
        self.tls_stream.write_all(request.body.as_slice()).await?;

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
        let mut body: Vec<u8> = Vec::new();
        if request.method == HttpMethod::Head {
            println!("no body for HEAD request");
            response.body = body.clone();
        } else if let Some(encoding) = response.headers.get("Transfer-Encoding")
            && encoding == "chunked"
        {
            println!("received chunked body");
            //todo
        } else {
            let body_length = response
                .headers
                .get("Content-Length")
                .ok_or_else(|| std::io::Error::other("missing Content-Length header"))?;
            println!("Content-Length = {}", body_length);
            let size: usize = body_length.parse().unwrap();
            for _ in 0..size {
                body.push(self.tls_stream.read_u8().await?);
            }
        }

        println!("{}", String::from_utf8(body.clone()).unwrap());
        println!("--------------------------------");
        response.body = body;

        Ok(response)
    }
}

#[derive(Display, Debug, PartialEq)]
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

const DOMAIN: &str = "gioyingtec.com";
const DOMAIN_CHUNK: &str = "anglesharp.azurewebsites.net";
const PORT: u16 = 443;

#[tokio::main]
async fn main() -> Result<()> {
    let mut http_connection = HttpConnection::new(DOMAIN.to_string(), PORT).await?;

    let request = HttpRequest::get(Url::parse("https://gioyingtec.com").unwrap());
    let response = http_connection.send(request).await?;

    let request = HttpRequest::get(Url::parse("https://gioyingtec.com").unwrap());
    let response = http_connection.send(request).await?;
    println!(
        "response body: \n{}",
        String::from_utf8(response.body).unwrap()
    );

    http_connection = HttpConnection::new(DOMAIN_CHUNK.to_string(), PORT).await?;

    let request =
        HttpRequest::get(Url::parse("https://anglesharp.azurewebsites.net/Chunked").unwrap());
    let response = http_connection.send(request).await?;
    println!(
        "response body: \n{}",
        String::from_utf8(response.body).unwrap()
    );

    Ok(())
}
