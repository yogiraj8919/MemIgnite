use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use std::io::Write;

use std::{ time::Duration};
use crate::stats::Stats;


use crate::{ command::Command, parser, store::Store};

pub async fn handle_client(socket: TcpStream,store:Store,stats:Stats) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut line = String::new();

    
let banner = r#"
=============================================================
 ███╗   ███╗███████╗███╗   ███╗██╗ ██████╗ ███╗   ██╗██╗████████╗███████╗
 ████╗ ████║██╔════╝████╗ ████║██║██╔════╝ ████╗  ██║██║╚══██╔══╝██╔════╝
 ██╔████╔██║█████╗  ██╔████╔██║██║██║  ███╗██╔██╗ ██║██║   ██║   █████╗
 ██║╚██╔╝██║██╔══╝  ██║╚██╔╝██║██║██║   ██║██║╚██╗██║██║   ██║   ██╔══╝
 ██║ ╚═╝ ██║███████╗██║ ╚═╝ ██║██║╚██████╔╝██║ ╚████║██║   ██║   ███████╗
 ╚═╝     ╚═╝╚══════╝╚═╝     ╚═╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚═╝   ╚═╝   ╚══════╝

                  MemIgnite v0.1.0
          In-Memory High Performance KV Engine
=============================================================

Type 'help' to see available commands.

"#;
    writer.write(banner.as_bytes()).await?;

    loop {
        
        line.clear();
        writer.write_all("🧠 MemIgnite>".as_bytes()).await?;
        

        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            // client closed connection
            break;
        }
        stats.incr_commands(); 

        let cmd = parser::parse_command(&line);

        match cmd {
            Command::Set { key, value,ex,.. } =>{
                let ttl = ex.map(Duration::from_secs);

                store.set(key.clone(),value.clone(),ttl).await;
               
                writer.write_all(b"OK\n").await?;
            }
            Command::Get { key } =>{
                if let Some(val) = store.get(&key).await{
                    writer.write_all(val.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                }else{
                    writer.write_all(b"{nil}\n").await?;
                }
            }
            Command::Del { key } => {
                let deleted = store.del(&key).await;
                 writer.write_all(format!("{}\n", deleted as u8).as_bytes()).await?;
            }
            Command::Help =>{
                let help_text = r#"
==================== MemIgnite Command Reference ====================

PING
    → Check server availability

ECHO <message>
    → Echo back the provided message

SET <key> <value> [EX <seconds>] 
    → Set a key with optional expiration

GET <key>
    → Retrieve value of a key

DEL <key>
    → Delete a key

HELP
    → Show this help message

QUIT
    → Close the connection

======================================================================

"#;
                writer.write_all(help_text.as_bytes()).await?;
            }
            Command::Ping => {
                writer.write_all(b"PONG\n").await?;
            }
            Command::Echo(msg) => {
                writer.write_all(msg.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
            Command::Quit => {
                writer.write_all(b"BYE\n").await?;
                break;
            }
            Command::LPUSH { key, value } =>{
                let len = store.lpush(key, value).await;
                writer.write_all(format!("{}\n",len).as_bytes()).await?;
            }
            Command::RDROP { key }=>{
                if let Some(val) = store.rdrop(&key).await{
                    writer.write_all(val.as_bytes()).await?;
                    writer.write_all(b"\n").await?;
                }else{
                     writer.write_all(b"{nil}\n").await?;
                }
            }
            Command::RewriteAof => {
                let mut aof = store.aof.lock().await;
                aof.writer.flush()?; 
                drop(aof);  
                crate::aof::Aof::rewrite(&store)?;
                let mut aof = store.aof.lock().await;
                aof.reopen()?;
                stats.incr_rewrite(); 
                writer.write_all(b"AOF rewrite completed\n").await?;
            }
            Command::Info => {
                let total_keys = store.inner.len();

                let mut string_keys = 0;
                let mut list_keys = 0;
                let mut expiring_keys = 0;

                for item in store.inner.iter(){
                    let entry = item.value();

                    if entry.expires_at.is_some(){
                        expiring_keys += 1;
                    }

                    match &entry.value {
                        crate::store::Value::String(_) => {string_keys += 1},
                        crate::store::Value::List(_) => {list_keys += 1}
                    }
                }

                let aof_size = std::fs::metadata("appendonly.aof")
                .map(|m| m.len())
                .unwrap_or(0);

                let info = format!(
                                     "\n# MemIgnite Stats\n\
                                     uptime: {}s\n\
                                     total_keys: {}\n\
                                     string_keys: {}\n\
                                     list_keys: {}\n\
                                     expiring_keys: {}\n\
                                     commands_processed: {}\n\
                                     clients_connected: {}\n\
                                     aof_size: {} bytes\n\
                                     rewrite_count: {}\n\n",
                                             stats.uptime(),
                                             total_keys,
                                             string_keys,
                                             list_keys,
                                             expiring_keys,
                                             stats.commands_processed.load(std::sync::atomic::Ordering::Relaxed),
                                             stats.clients_connected.load(std::sync::atomic::Ordering::Relaxed),
                                             aof_size,
                                             stats.rewrite_count.load(std::sync::atomic::Ordering::Relaxed),
                );

            writer.write_all(info.as_bytes()).await?;

                
            }
            Command::Unknown(name) => {
                writer
                    .write_all(format!("ERR unknown command: {}\n", name).as_bytes())
                    .await?;
            }
        }
    }

    Ok(())
}
