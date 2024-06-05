use std::{net::SocketAddr, sync::Arc};

use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::warn;

use crate::Message;

#[derive(Debug)]
pub struct Peer {
    pub username: String,
    pub stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[derive(Debug, Default)]
pub struct State {
    pub peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

impl State {
    pub async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() != &addr {
                if let Err(e) = peer.value().send(message.clone()).await {
                    warn!("failed to broadcast message to {}: {}", peer.key(), e);
                    self.peers.remove(peer.key());
                }
            }
        }
    }
    pub async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(128);
        self.peers.insert(addr, tx);
        let (mut sender, receiver) = stream.split();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = sender.send(message.to_string()).await {
                    warn!("failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });
        Peer {
            username,
            stream: receiver,
        }
    }
}
