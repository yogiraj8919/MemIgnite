mod server;
mod handler;
mod command;
mod parser;
mod store;
mod aof;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:6379";
    println!("MiniRedis is Listening on {}",addr);

    if let Err(e) = server::run(addr).await{
        eprint!("Server error: {}",e);
    }
}
