use anyhow::Result;
use clap::Parser;
use st_tcp_server::server::Server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = "3000")]
    port: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();
    let addr = format!("localhost:{}", args.port);
    let mut server = Server::new(&addr).await?;

    if let Err(e) = server.start().await {
        eprintln!("Error starting server: {:?}", e);
    }
    Ok(())
}
