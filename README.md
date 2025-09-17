<h1 align="center">async_tiny</h1>
<p align="center">
  <a href="https://crates.io/crates/async_tiny">
    <img src="https://img.shields.io/crates/v/async_tiny?style=flat-square" alt="Crates.io">
  </a>
  <a href="https://docs.rs/async_tiny">
    <img src="https://img.shields.io/docsrs/async_tiny?style=flat-square" alt="Docs.rs">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT">
  </a>
</p>

**A minimal async HTTP server with a `tiny_http`-like feel, built on Hyper 1.x.**

`async_tiny` is designed for simplicity: it gives you a clean, buffered request loop without exposing Hyper internals or requiring complex async plumbing. Ideal for small web apps, embedded tools, or frameworks like [Velto](https://github.com/pjdur/velto).

---

## ✨ Features

- Async HTTP/1.1 server powered by Hyper
- Fully buffered request bodies (`Bytes`)
- Simple `Request` and `Response` types
- Clean loop: `while let Some(req) = server.next().await`
- Respond via `req.respond(Response)`
- No Hyper types exposed across threads
- Optional silent mode for clean logging

---

## 🚀 Quick Start

```rust
use async_tiny::{Server, Response};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::http("127.0.0.1:8080", false).await?;

    while let Some(request) = server.next().await {
        let response = Response::from_string("Hello from async_tiny!");
        let _ = request.respond(response);
    }

    Ok(())
}
```
> ⚠️ **Note:** If you want to use `#[tokio::main]` in your own code, you must add Tokio to your project manually:
> ```bash
> cargo add tokio
> ```
> Although this crate depends on Tokio internally, Rust requires that procedural macros like `#[tokio::main]` be declared directly in your own Cargo.toml to work properly.
---

## 📦 Request API

```rust
struct Request {
    fn url(&self) -> &str
    fn method(&self) -> &Method
    fn headers(&self) -> &HeaderMap
    fn body(&self) -> &Bytes
    fn respond(self, Response) -> Result<(), RespondError>
}
```

---

## 📤 Response API

```rust
Response::from_string("Hello")
Response::from_data(vec![1, 2, 3])
Response::from_status_and_string(404, "Not Found")
Response::empty(204)
    .with_content_type("text/plain")
    .with_header(Header::from_str("X-Custom: Value")?)
```

---

## 🔧 Silent Mode

Suppress internal logging (e.g. connection errors, startup messages):

```rust
let server = Server::http("127.0.0.1:8080", true).await?;
```

---

## 🛠 Used By

- [Velto](https://github.com/pjdur/velto) — a minimal async web framework with LiveReload and templating.

---

## 📚 License

MIT

---

## 💬 Feedback

Open an issue or reach out via [GitHub Discussions](https://github.com/pjdur/async_tiny/discussions) if you have ideas, bugs, or suggestions.
