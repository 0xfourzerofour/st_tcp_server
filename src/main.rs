use anyhow::Result;
use server::Server;
use tokio::net::TcpListener;

mod client;
mod events;
mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let addr = "localhost:3000";
    let listner = TcpListener::bind(addr).await?;
    let mut server = Server::new(listner);

    match server.start().await {
        Ok(()) => {
            println!("Server started successfully on {}.", addr);
        }
        Err(e) => {
            eprintln!("Error starting server: {:?}", e);
        }
    }
    Ok(())
}
