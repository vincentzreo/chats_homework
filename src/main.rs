use std::{net::SocketAddr, sync::Arc};

use chats::{Message, State};
use futures::{SinkExt as _, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    let state = Arc::new(State::default());
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("accepted connection from {}", addr);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(addr, stream, state_clone).await {
                warn!("failed to handle client: {}", e);
            }
        });
    }
}

async fn handle_client(
    addr: SocketAddr,
    stream: TcpStream,
    state: Arc<State>,
) -> anyhow::Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;
    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };
    let mut peer = state.add(addr, username, stream).await;
    let message = Arc::new(Message::user_joined(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("failed to read line from {}: {}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(&peer.username, line));
        info!("{}", message);
        state.broadcast(addr, message).await;
    }
    let message = Arc::new(Message::user_left(&peer.username));
    info!("{}", message);
    state.broadcast(addr, message).await;
    Ok(())
}
