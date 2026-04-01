mod server;
mod handler;
mod command;
mod parser;
mod store;
mod aof;
mod stats;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::aof::{ Aof,FsyncPolicy};
use crate::store::Store;

#[tokio::main]
async fn main() {
    print_banner();

    let addr = "127.0.0.1:6379";
    println!("MemIgnite is Listening on {}", addr);

    let aof = Arc::new(Mutex::new(Aof::new(FsyncPolicy::EverySec)));
    let store = Store::new(aof.clone());

    store.clone().start_expiration_task();
    let stats = stats::Stats::new();

    if let Err(e) = server::run(addr, store,stats).await {
        eprintln!("Server error: {}", e);
    }
}

fn print_banner() {
    println!();
    println!("🧠  MemIgnite v0.1.0");
    println!("    In-Memory Key-Value Engine");
    println!("------------------------------------------------");
    println!();
}