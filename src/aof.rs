

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};




#[allow(unused)]

#[derive(Clone, Copy)]
pub enum FsyncPolicy {
    Always,
    EverySec,
    No 
}


pub struct Aof{
    pub writer:BufWriter<File>,
    policy:FsyncPolicy,
    last_sync:Instant
}

impl Aof {
    pub fn new(policy: FsyncPolicy)->Self{
        let  file = OpenOptions::new().create(true)
        .append(true)
        .open("appendonly.aof")
        .unwrap();

    Aof { writer: BufWriter::new(file), policy, last_sync: Instant::now() }
    }

    pub fn append(&mut self, cmd: &str)-> std::io::Result<()>{
        writeln!(self.writer, "{}",cmd)?;

        match self.policy {
            FsyncPolicy::Always => {
                self.writer.flush()?;
            }
            FsyncPolicy::EverySec=>{
                if self.last_sync.elapsed() >= Duration::from_secs(1){
                    self.writer.flush()?;
                    self.last_sync = Instant::now();
                }
            }
            FsyncPolicy::No => {
                
            }
        }
        Ok(())
    }

    

    pub fn rewrite_from_snapshot(snapshot:Vec<(String,crate::store::Entry)>) -> std::io::Result<()>{
        use std::fs::File;
        use std::io::{BufWriter,Write};

        let temp_file = File::create("temp-rewrite.aof")?;
        let mut writer = BufWriter::new(temp_file);

        for (key,entry) in snapshot{
            match entry.value {
                crate::store::Value::String(val) => {
                    if let Some(expire) = entry.expires_at{
                        writeln!(writer,"SET {} {} EXAT {}", key, val, expire)?;
                    }else {
                        writeln!(writer, "SET {} {}", key, val)?;
                    }
                }
                crate::store::Value::List(list) =>{
                    for value in list{
                        writeln!(writer,"LPUSH {} {}",key,value)?;
                    }
                }
            }
        }
        writer.flush()?;
        std::fs::rename("temp-rewrite.aof", "appendonly.aof")?;
        Ok(())
    }
}