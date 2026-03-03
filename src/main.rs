mod server;
mod handler;
mod command;
mod parser;
mod store;
mod aof;

use store::Store;

#[tokio::main]
async fn main() {
    print_banner();

    let addr = "127.0.0.1:6379";
    println!("MemIgnite is Listening on {}", addr);


    let store = Store::new();


    store.clone().start_expiration_task();

    if let Err(e) = server::run(addr, store).await {
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