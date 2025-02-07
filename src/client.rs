use crate::events::Event;
use anyhow::Result;
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, ReadHalf, WriteHalf},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Client {
    pub id: String,
    reader: Lines<BufReader<ReadHalf<TcpStream>>>,
    writer: WriteHalf<TcpStream>,
    is_terminated: bool,
}

impl Stream for Client {
    type Item = Event;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.is_terminated {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.reader).poll_next_line(cx) {
            Poll::Ready(Ok(line)) => match line {
                Some(l) => Poll::Ready(Some(Event::Message(self.id.clone(), l))),
                None => {
                    self.is_terminated = true;
                    Poll::Ready(Some(Event::Disconnect(self.id.clone())))
                }
            },
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(_)) => {
                self.is_terminated = true;
                Poll::Ready(Some(Event::Disconnect(self.id.clone())))
            }
        }
    }
}

impl Client {
    pub fn new(stream: TcpStream, id: String) -> Self {
        let (read, writer) = split(stream);
        let reader = BufReader::new(read).lines();
        Self {
            id,
            writer,
            reader,
            is_terminated: false,
        }
    }
    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        self.writer.write_all(message.as_bytes()).await?;
        Ok(())
    }
}
