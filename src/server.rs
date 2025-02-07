use crate::client::Client;
use crate::events::Event;
use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamFuture};
use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;

#[derive(Debug)]
pub struct Server {
    listener_stream: TcpListenerStream,
    client_futures: FuturesUnordered<StreamFuture<Client>>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        let listener_stream = TcpListenerStream::new(listener);

        Ok(Self {
            listener_stream,
            client_futures: FuturesUnordered::new(),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                new_connection = self.listener_stream.next() => {
                    if let Some(result) = new_connection {
                        let stream = result?;
                        self.handle_new_connection(stream).await?;
                    }
                },
                Some((event, client)) = self.client_futures.next() => {
                    self.handle_event(event, client).await?;
                }
            }
        }
    }

    async fn handle_new_connection(&mut self, stream: TcpStream) -> Result<()> {
        let socket = stream.peer_addr()?;
        let ip = socket.ip().to_string();
        let port = socket.port().to_string();
        println!("connected {ip} {port}");
        let mut client = Client::new(stream, port.clone());
        let event = Event::Login(port);
        client.send_message(&event.to_string()).await?;
        self.client_futures.push(client.into_future());
        Ok(())
    }

    async fn handle_event(&mut self, event: Option<Event>, mut client: Client) -> Result<()> {
        match event {
            Some(e) => match e {
                Event::Disconnect(client_id) => {
                    println!("disconnected {}", client_id);
                }
                Event::Message(_, _) => {
                    for c in &mut self.client_futures {
                        if let Some(cl) = c.get_mut() {
                            cl.send_message(&e.to_string()).await?;
                        }
                    }
                    client.send_message("ACK:MESSAGE\n").await?;
                    self.client_futures.push(client.into_future());
                }
                _ => {}
            },
            None => {}
        }
        Ok(())
    }
}
