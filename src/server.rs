use crate::client::Client;
use crate::events::Event;
use anyhow::Result;
use futures::task::noop_waker_ref;
use std::collections::{HashMap, VecDeque};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    clients: HashMap<String, Client>,
    event_queue: VecDeque<Event>,
    poll_rate: Duration,
}

impl Server {
    pub async fn new(addr: &str, poll_rate: u64) -> Result<Self> {
        let listener = TcpListener::bind(&addr).await?;

        Ok(Self {
            listener,
            clients: HashMap::new(),
            event_queue: VecDeque::new(),
            poll_rate: Duration::from_millis(poll_rate),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        println!("listening on port {}", self.listener.local_addr()?.port());
        self.poll_connection_and_process().await
    }

    async fn poll_connection_and_process(&mut self) -> Result<()> {
        let mut context = Context::from_waker(noop_waker_ref());
        loop {
            match self.listener.poll_accept(&mut context) {
                Poll::Ready(Ok((socket, addr))) => {
                    let client_id = addr.port().to_string();
                    println!("connected {} {}", addr.ip(), client_id);
                    let client = Client::new(socket, client_id.clone());
                    self.event_queue.push_back(Event::Login(client_id.clone()));
                    self.clients.insert(client_id, client);
                }
                Poll::Pending => {
                    self.check_and_process_clients().await?;
                    // pause main thread to configurable length so that we do not
                    // overload the cpu with io operations
                    sleep(self.poll_rate).await;
                }
                Poll::Ready(Err(e)) => {
                    return Err(e.into());
                }
            }
        }
    }

    async fn check_and_process_clients(&mut self) -> Result<()> {
        let clients: Vec<(&String, &mut Client)> = self.clients.iter_mut().collect();
        // check for new messages
        for (_id, client) in clients {
            if let Some(event) = client.read_line().await? {
                self.event_queue.push_back(event);
            }
        }

        // process received messages
        while let Some(event) = self.event_queue.pop_front() {
            self.handle_event(&event).await?;
        }

        Ok(())
    }

    async fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Login(client_id) | Event::Disconnect(client_id) => {
                if let Some(client) = self.clients.get_mut(client_id) {
                    client.send_message(&event.to_string()).await?;
                }
                if let Event::Disconnect(_) = event {
                    println!("client {} disconnected.", client_id);
                    self.clients.remove(client_id);
                }
            }
            Event::Message(from_id, content) => {
                if let Some(sender) = self.clients.get_mut(from_id) {
                    sender.send_message("ACK:MESSAGE\n").await?;
                }

                println!("message {} {}", from_id, content);

                let clients_to_broadcast: Vec<(&String, &mut Client)> = self
                    .clients
                    .iter_mut()
                    .filter(|(id, _)| *id != from_id)
                    .collect();

                for (id, client) in clients_to_broadcast {
                    if let Err(e) = client.send_message(&event.to_string()).await {
                        eprintln!("Failed to send message to {}: {}", id, e);
                    }
                }
            }
        }
        Ok(())
    }
}
