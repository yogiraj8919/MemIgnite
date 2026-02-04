// Store module for mini_redis

use std::{ collections::HashMap, sync::Arc, time::{Instant,Duration}};




use tokio::sync::RwLock;


#[derive(Clone)]
pub struct Store{
    inner:Arc<RwLock<HashMap<String,Entry>>>
}

struct Entry{
    value:String,
    expires_at:Option<Instant>
}


impl Store {
     pub fn new() -> Self{
        Store{
            inner : Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub async fn set(&self,key:String,value:String,ttl:Option<Duration>){
        let expires_at = ttl.map(|d| Instant::now() + d);
        let mut map  = self.inner.write().await;
        map.insert(key,Entry { value, expires_at });
    }

    pub async fn get(&self,key:&str) -> Option<String>{
        let mut  map = self.inner.write().await;
        if let Some(entry) = map.get(key){
            if let Some(expire) = entry.expires_at{
                if Instant::now() > expire{
                    //expired -> delete
                    map.remove(key);
                    return None;
                }
               
            }
            return Some(entry.value.clone());
        }
        None
    }

    pub async fn del(&self, key:&str) -> bool{
        let mut map = self.inner.write().await;
        map.remove(key).is_some()
    }

    pub async fn apply_raw(&self,input:&str){
        use crate::parser::parse_command;
        use crate::command::Command;

        match parse_command(input) {
            Command::Set { key, value, ex } => {
                let ttl = ex.map(Duration::from_secs);
                self.set(key, value, ttl).await;
            }
            Command::Del { key }=> {
                self.del(&key).await;
            }
            _ =>{}
        }

    }

}