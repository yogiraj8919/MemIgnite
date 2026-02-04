use std::sync::Arc;

// Server module for mini_redis
use tokio::{net::TcpListener, sync::Mutex};
use crate::{aof::{Aof}, handler, store::Store};



pub async fn run(addr : &str) -> Result<(),Box<dyn std::error::Error>>{
    let listener = TcpListener::bind(addr).await?;
    let store = Store::new();
    let aof = Arc::new(Mutex::new(Aof::new("appendonly.aof")?));

    
    // replay persisted commands
    let cmds = Aof::load("appendonly.aof")?;
    for cmd in cmds {
        store.apply_raw(&cmd).await;
    }

    loop{
        let (socket, peer_addr) = listener.accept().await?;
        let store = store.clone();
        let aof = aof.clone();


        println!("Client connected: {}",peer_addr);

        tokio::spawn(async move{
            handler::handle_client(socket, store, aof).await.ok();
        });

    }
}