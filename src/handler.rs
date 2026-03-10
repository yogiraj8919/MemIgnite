use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use tokio::sync::Mutex;

use std::{ sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use crate::{aof::Aof, command::Command, parser, store::Store};

pub async fn handle_client(socket: TcpStream,store:Store,aof:Arc<Mutex<Aof>>) -> Result<(), Box<dyn std::error::Error>> {
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

        let cmd = parser::parse_command(&line);

        match cmd {
            Command::Set { key, value,ex,.. } =>{
                let ttl = ex.map(Duration::from_secs);
                store.set(key.clone(),value.clone(),ttl).await;
                let mut aof = aof.lock().await;
                if let Some(sec) = ex{
                    let now = SystemTime::now()
                       .duration_since(UNIX_EPOCH)
                       .unwrap()
                       .as_secs();
                    let expiry_ts = now + sec;
                     aof.append(&format!("SET {} {} EXAT {}",key,value,expiry_ts))?;
                }else {
                     aof.append(&format!("SET {} {}", key, value))?;
                }
               
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
                if deleted{
                    let mut aof = aof.lock().await;
                    aof.append(&format!("DEL {}",key))?;
                }
                writer.write_all(format!("{}\n",deleted as u8).as_bytes()).await?;
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
            Command::Unknown(name) => {
                writer
                    .write_all(format!("ERR unknown command: {}\n", name).as_bytes())
                    .await?;
            }
        }
    }

    Ok(())
}
