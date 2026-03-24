

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

use crate::store::{Store, Value};

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
    pub fn rewrite(store: &Store) -> std::io::Result<()>{

        let file = File::create("temp-rewrite.aof")?;
        let mut writer = BufWriter::new(file);

        for entry in store.inner.iter() {
            let key = entry.key();
            let val = entry.value();

            match &val.value {
                Value::String(v) => {
                    if let Some(exp) = val.expires_at {
                        writeln!(writer,"SET {} {} EXAT {}",key,v,exp)?;
                    }else {
                        writeln!(writer,"SET {} {}",key,v)?;
                    }
                }
                Value::List(list) => {
                    for item in list{
                        writeln!(writer,"LPUSH {} {}",key,item)?;
                    }
                }
            }
        }

        writer.flush()?;

        std::fs::rename("temp-rewrite.aof", "appendonly.aof")?;

        Ok(())

    }

    pub fn reopen(&mut self) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("appendonly.aof")?;

        self.writer = BufWriter::new(file);
        Ok(())
    }
}