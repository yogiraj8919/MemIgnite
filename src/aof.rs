use std::fs::{OpenOptions, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;


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


}
