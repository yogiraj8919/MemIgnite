// Store module for mini_redis
use tokio::time::interval;
use std::{ collections::{HashMap}, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};




use tokio::sync::RwLock;


#[derive(Clone)]
pub struct Store{
    inner:Arc<RwLock<HashMap<String,Entry>>>
}

struct Entry{
    value:String,
    expires_at:Option<u64>
}


impl Store {
     pub fn new() -> Self{
        Store{
            inner : Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub async fn set(&self,key:String,value:String,ttl:Option<Duration>){
        let mut map = self.inner.write().await;

        let expires_at = ttl.map(|d| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + d.as_secs()
        });
     
        map.insert(key,Entry { value, expires_at });
    }

    pub async fn get(&self,key:&str) -> Option<String>{
        let mut  map = self.inner.write().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
           .as_secs();
        
        if let Some(entry) = map.get(key){
            if let Some(expire) = entry.expires_at{
                if now >= expire{
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

    pub fn start_expiration_task(self){
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop{
                ticker.tick().await;
                let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut map = self.inner.write().await;

            let key_to_remove:Vec<String> = map.iter().filter_map(|(key,entry)|{
                if let Some(expire) = entry.expires_at{
                    if now >= expire{
                        return Some(key.clone());
                    }
                }
                None
            })
            .collect();
        for key in key_to_remove {
            map.remove(&key);
        }
            }
        });
    }

    pub async fn apply_raw(&self,input:&str){
        use crate::parser::parse_command;
        use crate::command::Command;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(); 


        match parse_command(input) {

            Command::Set { key, value, exat,.. } => {
               if let Some(expity_ts) = exat {
                   if expity_ts <= now{
                    return;
                   }

                   let remaining = expity_ts - now;
                   self.set(key, value, Some(Duration::from_secs(remaining)))
                   .await;
               }else {
                   self.set(key, value, None).await;
               }
               

            }
            Command::Del { key }=> {
                self.del(&key).await;
            }
            _ =>{}
        }

    }

}