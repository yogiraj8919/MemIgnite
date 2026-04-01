

use std::{fs::File, io::{BufRead, BufReader}};

use tokio::{net::TcpListener};
use crate::{ handler, store::Store};
use crate::stats::Stats;



pub async fn run(addr : &str, store:Store, stats:Stats) -> Result<(),Box<dyn std::error::Error>>{
    let listener = TcpListener::bind(addr).await?;


    
    // replay persisted commands
   
if let Ok(file) = File::open("appendonly.aof") {
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(cmd) = line {
            if !cmd.trim().is_empty() {
                store.apply_raw(&cmd).await;
            }
        }
    }
}

    loop{
        let (socket, peer_addr) = listener.accept().await?;
        let store = store.clone();
        let stats = stats.clone();  
        stats.incr_clients();
        

        println!("Client connected: {}",peer_addr);

        tokio::spawn(async move{
            handler::handle_client(socket, store,stats).await.ok();
        });

    }
}