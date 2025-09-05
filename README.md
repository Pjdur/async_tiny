<p align="center">

&nbsp; <h1 align="center">async\_tiny</h1>

&nbsp; <p align="center">

&nbsp;   <a href="https://crates.io/crates/async\_tiny">

&nbsp;     <img src="https://img.shields.io/crates/v/async\_tiny?style=flat-square" alt="Crates.io">

&nbsp;   </a>

&nbsp;   <a href="https://docs.rs/async\_tiny">

&nbsp;     <img src="https://img.shields.io/docsrs/async\_tiny?style=flat-square" alt="Docs.rs">

&nbsp;   </a>

&nbsp;   <a href="https://opensource.org/licenses/MIT">

&nbsp;     <img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License: MIT">

&nbsp;   </a>

&nbsp; </p>

</p>



\*\*A minimal async HTTP server with a `tiny\_http`-like feel, built on Hyper 1.x.\*\*



`async\_tiny` is designed for simplicity: it gives you a clean, buffered request loop without exposing Hyper internals or requiring complex async plumbing. Ideal for small web apps, embedded tools, or frameworks like \[Velto](https://github.com/pjdur/velto).



---



\## ✨ Features



\- Async HTTP/1.1 server powered by Hyper

\- Fully buffered request bodies (`Bytes`)

\- Simple `Request` and `Response` types

\- Clean loop: `while let Some(req) = server.next().await`

\- Respond via `req.respond(Response)`

\- No Hyper types exposed across threads

\- Optional silent mode for clean logging



---



\## 🚀 Quick Start



```rust

use async\_tiny::{Server, Response};



\#\[tokio::main]

async fn main() -> std::io::Result<()> {

&nbsp;   let mut server = Server::http("127.0.0.1:8080", false).await?;



&nbsp;   while let Some(request) = server.next().await {

&nbsp;       let response = Response::from\_string("Hello from async\_tiny!");

&nbsp;       let \_ = request.respond(response);

&nbsp;   }



&nbsp;   Ok(())

}

```



---



\## 📦 Request API



```rust

struct Request {

&nbsp;   fn url(\&self) -> \&str

&nbsp;   fn method(\&self) -> \&Method

&nbsp;   fn headers(\&self) -> \&HeaderMap

&nbsp;   fn body(\&self) -> \&Bytes

&nbsp;   fn respond(self, Response) -> Result<(), RespondError>

}

```



---



\## 📤 Response API



```rust

Response::from\_string("Hello")

Response::from\_data(vec!\[1, 2, 3])

Response::from\_status\_and\_string(404, "Not Found")

Response::empty(204)

&nbsp;   .with\_content\_type("text/plain")

&nbsp;   .with\_header(Header::from\_str("X-Custom: Value")?)

```



---



\## 🔧 Silent Mode



Suppress internal logging (e.g. connection errors, startup messages):



```rust

let server = Server::http("127.0.0.1:8080", true).await?;

```



---



\## 🛠 Used By



\- \[Velto](https://github.com/pjdur/velto) — a minimal async web framework with LiveReload and templating.



---



\## 📚 License



MIT



---



\## 💬 Feedback



Open an issue or reach out via \[GitHub Discussions](https://github.com/pjdur/async\_tiny/discussions) if you have ideas, bugs, or suggestions.



