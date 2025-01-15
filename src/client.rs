use std::{
    pin::Pin,
    task::{Context, Poll},
};

use ::futures::task::noop_waker_ref;
use anyhow::Result;
use tokio::{
    io::{
        split, AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, ReadBuf, ReadHalf, WriteHalf,
    },
    net::TcpStream,
};

use crate::events::Event;

#[derive(Debug)]
pub struct Client {
    id: String,
    reader: BufReader<ReadHalf<TcpStream>>,
    writer: WriteHalf<TcpStream>,
}

impl Client {
    pub fn new(stream: TcpStream, id: String) -> Self {
        let (read, writer) = split(stream);
        let reader = BufReader::new(read);
        Self { id, writer, reader }
    }

    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        self.writer.write_all(message.as_bytes()).await?;
        Ok(())
    }

    // using polling to check if the data is ready to read so that
    // we do not get blocked on the read_line function
    pub async fn read_line(&mut self) -> Result<Option<Event>> {
        let mut buf = String::new();
        let mut context = Context::from_waker(noop_waker_ref());

        match Pin::new(&mut self.reader).poll_read(&mut context, &mut ReadBuf::new(&mut [])) {
            Poll::Ready(Ok(_)) => match self.reader.read_line(&mut buf).await {
                Ok(bytes) if bytes > 0 => {
                    let line = buf.trim().to_string();
                    Ok(Some(Event::Message(self.id.clone(), line)))
                }
                _ => Ok(Some(Event::Disconnect(self.id.clone()))),
            },
            Poll::Pending => Ok(None),
            Poll::Ready(Err(_)) => Ok(Some(Event::Disconnect(self.id.clone()))),
        }
    }
}
