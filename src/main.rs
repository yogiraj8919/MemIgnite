mod server;
mod handler;
mod command;
mod parser;
mod store;
mod aof;
mod stats;

use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::aof::{ Aof,FsyncPolicy};
use crate::store::Store;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    print_banner();

    let addr = "127.0.0.1:6379";
    println!("MemIgnite is Listening on {}", addr);

    let aof = Arc::new(Mutex::new(Aof::new(FsyncPolicy::EverySec)));
    let (aof_tx, mut aof_rx) = mpsc::unbounded_channel::<String>();

    let aof_clone = aof.clone();

    tokio::spawn(async move{
        while let Some(cmd) = aof_rx.recv().await{
            let mut aof = aof_clone.lock().await;
            aof.append(&cmd).ok();
        }
    });

    let store= Store::new(aof.clone(), aof_tx);
    store.clone().start_expiration_task();

    let stats = stats::Stats::new();

    tokio::select! {
        result = server::run(addr,store.clone(),stats) => {
            if let Err(e) = result{
                eprintln!("Server error: {}", e);
            }
        },

        _ = tokio::signal::ctrl_c() => {
            println!("\n Graceful shutdown started...");
        }
    }

    {
        let mut aof = aof.lock().await;
        aof.writer.flush().ok();
    }

    println!("🧠 MemIgnite preserved state and went offline cleanly.");
}

fn print_banner() {
    println!();
    println!("🧠  MemIgnite v0.1.0");
    println!("    In-Memory Key-Value Engine");
    println!("------------------------------------------------");
    println!();
}