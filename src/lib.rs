//! `async_tiny` is a minimal async HTTP server with a tiny_http-like feel, built on Hyper 1.x.
//!
//! Designed for simplicity and clarity, it lets you handle HTTP requests using a clean,
//! synchronous-style loop—without exposing Hyper internals or requiring complex async plumbing.
//!
//! ## How it works
//! - Hyper accepts connections and parses requests.
//! - Each request body is fully buffered into `Bytes`.
//! - A simplified `Request` (method, headers, URL, body) is sent over an `mpsc` channel.
//! - You receive it via `Server::next().await` and respond using `req.respond(Response)`.
//! - The response is translated back into Hyper and sent to the client.
//!
//! This design avoids sending Hyper types across threads and keeps everything `Send`.
//! It's ideal for small web apps, embedded tools, or frameworks like [Velto](https://github.com/pjdur/velto).

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;

use bytes::Bytes;
use http::{HeaderMap, Method, StatusCode, Uri};
pub use http::{HeaderName, HeaderValue};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming as HyperBody;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

/// The main server: bind with Server::http(...).await?, then loop server.next().await.
pub struct Server {
    rx: mpsc::Receiver<Request>,
    _join: tokio::task::JoinHandle<()>,
}

impl Server {
    /// Bind an HTTP/1 server on addr like "127.0.0.1:8080".
    pub async fn http(addr: &str, silent: bool) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel::<Request>(1024);
        let addr: SocketAddr = addr.parse().map_err(into_io_error)?;

        let tx_clone = tx.clone();

        let join = tokio::spawn({
            let silent = silent;
            async move {
                let listener = TcpListener::bind(addr).await.expect("bind failed");
                if !silent {
                    eprintln!("async_tiny listening on http://{}", addr);
                }

                loop {
                    let (stream, _) = match listener.accept().await {
                        Ok(s) => s,
                        Err(e) => {
                            if !silent {
                                eprintln!("Accept error: {}", e);
                            }
                            continue;
                        }
                    };

                    let io = TokioIo::new(stream);
                    let tx = tx_clone.clone();

                    tokio::spawn(async move {
                        let service =
                            hyper::service::service_fn(move |req: HyperRequest<HyperBody>| {
                                let tx = tx.clone();
                                async move {
                                    let url = path_and_query(req.uri());
                                    let (parts, body) = req.into_parts();
                                    let collected = match body.collect().await {
                                        Ok(c) => c.to_bytes(),
                                        Err(_) => Bytes::new(),
                                    };

                                    let (resp_tx, resp_rx) = oneshot::channel::<Response>();

                                    let request = Request {
                                        method: parts.method,
                                        headers: parts.headers,
                                        url,
                                        body: collected,
                                        respond_tx: Some(resp_tx),
                                    };

                                    if tx.send(request).await.is_err() {
                                        return Ok::<_, Infallible>(response_text(
                                            StatusCode::SERVICE_UNAVAILABLE,
                                            "Service Unavailable",
                                        ));
                                    }

                                    let resp = match resp_rx.await {
                                        Ok(r) => to_hyper_response(r),
                                        Err(_) => response_text(
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            "Internal Server Error",
                                        ),
                                    };

                                    Ok::<_, Infallible>(resp)
                                }
                            });

                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, service)
                            .await
                        {
                            if !silent {
                                eprintln!("Connection error: {:?}", err);
                            }
                        }
                    });
                }
            }
        });

        Ok(Server { rx, _join: join })
    }

    /// Await the next incoming request from any connection.
    pub async fn next(&mut self) -> Option<Request> {
        self.rx.recv().await
    }
}

/// A tiny_http-like request handed to your loop.
pub struct Request {
    method: Method,
    headers: HeaderMap,
    url: String,
    body: Bytes,
    respond_tx: Option<oneshot::Sender<Response>>,
}

impl Request {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn respond(mut self, response: Response) -> Result<(), RespondError> {
        let tx = self
            .respond_tx
            .take()
            .ok_or(RespondError::AlreadyResponded)?;
        tx.send(response).map_err(|_| RespondError::ChannelClosed)
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        if let Some(tx) = self.respond_tx.take() {
            let _ = tx.send(Response::from_status_and_string(500, "No response"));
        }
    }
}

#[derive(Debug)]
pub enum RespondError {
    AlreadyResponded,
    ChannelClosed,
}

/// A tiny response wrapper (status, headers, body).
#[derive(Clone)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl Response {
    pub fn from_data(data: impl Into<Bytes>) -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: data.into(),
        }
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self::from_data(Bytes::from(s.into()))
    }

    pub fn from_status_and_string(code: u16, s: impl Into<String>) -> Self {
        let status = StatusCode::from_u16(code).unwrap_or(StatusCode::OK);
        Self {
            status,
            headers: HeaderMap::new(),
            body: Bytes::from(s.into()),
        }
    }

    pub fn empty(status: u16) -> Self {
        let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
        Self {
            status,
            headers: HeaderMap::new(),
            body: Bytes::new(),
        }
    }

    pub fn with_status_code(mut self, code: u16) -> Self {
        self.status = StatusCode::from_u16(code).unwrap_or(StatusCode::OK);
        self
    }

    pub fn with_header(mut self, header: Header) -> Self {
        self.headers.insert(header.0, header.1);
        self
    }

    pub fn with_content_type(self, value: &str) -> Self {
        let header =
            Header::from_str(&format!("Content-Type: {}", value)).expect("valid content type");
        self.with_header(header)
    }
}

/// A simple "Name: value" header wrapper (tiny_http style).
pub struct Header(pub HeaderName, pub HeaderValue);

impl Header {
    /// Creates a new Header from name and value strings.
    pub fn new(name: &str, value: &str) -> Result<Self, HeaderParseError> {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| HeaderParseError::InvalidName)?;
        let value = HeaderValue::from_str(value)
            .map_err(|_| HeaderParseError::InvalidValue)?;
        Ok(Header(name, value))
    }
}

#[derive(Debug)]
pub enum HeaderParseError {
    InvalidFormat,
    InvalidName,
    InvalidValue,
}

impl std::str::FromStr for Header {
    type Err = HeaderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s.splitn(2, ':');
        let name = it.next().ok_or(HeaderParseError::InvalidFormat)?.trim();
        let value = it.next().ok_or(HeaderParseError::InvalidFormat)?.trim();

        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|_| HeaderParseError::InvalidName)?;
        let value = HeaderValue::from_str(value).map_err(|_| HeaderParseError::InvalidValue)?;
        Ok(Header(name, value))
    }
}

fn path_and_query(uri: &Uri) -> String {
    match uri.path_and_query() {
        Some(pq) => pq.as_str().to_string(),
        None => uri.path().to_string(),
    }
}

fn to_hyper_response(r: Response) -> HyperResponse<Full<Bytes>> {
    let mut builder = HyperResponse::builder().status(r.status);
    {
        let headers = builder.headers_mut().expect("headers mut");
        for (name, value) in r.headers.iter() {
            headers.append(name.clone(), value.clone());
        }
    }
    builder.body(Full::new(r.body)).expect("response build")
}

fn response_text(status: StatusCode, text: &str) -> HyperResponse<Full<Bytes>> {
    let r = Response::from_status_and_string(status.as_u16(), text).with_header(Header(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("text/plain; charset=utf-8"),
    ));
    to_hyper_response(r)
}

fn into_io_error<E: std::fmt::Display>(e: E) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e))
}
