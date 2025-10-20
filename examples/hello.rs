use async_tiny::{Response, Server};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::http("127.0.0.1:8080", false).await?;

    println!("🚀 Hello server running at http://127.0.0.1:8080");

    while let Some(request) = server.next().await {
        let response = Response::from_string("Hello from async_tiny!");
        let _ = request.respond(response);
    }

    Ok(())
}
