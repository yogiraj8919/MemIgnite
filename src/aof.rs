use std::fs::{OpenOptions, File,rename};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;



use crate::store::{Store};


pub struct Aof{
    file:File
}

impl Aof {
    pub fn new(path:&str) -> std::io::Result<Self>{
        let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

        Ok(Aof { file })
    }

    pub fn append(&mut self, cmd:&str)->std::io::Result<()>{
        writeln!(self.file,"{}",cmd)?;
        self.file.flush()?;
        Ok(())
    }

    pub fn load(path: &str) -> std::io::Result<Vec<String>>{
        if !Path::new(path).exists(){
            return Ok(vec![]);
        }
        let file = File::open(path)?;
        let  reader = BufReader::new(file);
        let mut commands = Vec::new();
        for line in reader.lines() {
            commands.push(line?);
        }
        Ok(commands)
    }

    pub fn rewrite(store:&Store) -> std::io::Result<()>{
        let file = File::create("temp-rewrite.aof")?;
        let mut writer = BufWriter::new(file);

        for entry in store.inner.iter(){
            let key = entry.key();
            let val = entry.value();

            match &val.value{

                crate::store::Value::String(v) =>{
                    if let Some(exp) = val.expires_at{
                        writeln!(writer,"SET {} {} EXAT {}",key,v,exp)?;
                    }else {
                         writeln!(writer,"SET {} {}",key,v)?;
                    }
                }
                crate::store::Value::List(list) => {
                     for item in list {
                    writeln!(writer,"LPUSH {} {}",key,item)?;
                }
                }
            }
        }
        writer.flush()?;
        rename("temp-rewrite.aof","appendonly.aof")?;
        Ok(())
    }


}
