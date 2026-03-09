// Store module for mini_redis

use std::{ sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};

use tokio::time::interval;


use dashmap::{DashMap};



#[derive(Clone)]
pub struct Store{
    inner:Arc<DashMap<String,Entry>>
}

struct Entry{
    value:String,
    expires_at:Option<u64>
}


impl Store {
     pub fn new() -> Self{
        Store{
            inner : Arc::new(DashMap::new())
        }
    }

    pub async fn set(&self,key:String,value:String,ttl:Option<Duration>){
    

        let expires_at = ttl.map(|d| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + d.as_secs()
        });
     
        self.inner.insert(key,Entry { value, expires_at });
    }

    pub async fn get(&self,key:&str) -> Option<String>{

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
           .as_secs();
        
        if let Some(entry) = self.inner.get(key){
            if let Some(expire) = entry.expires_at{
                if now >= expire{
                    drop(entry); 
                    //expired -> delete
                    self.inner.remove(key);
                    return None;
                }
               
            }
  

            return Some(entry.value.clone());
        }
        None
    }

    pub async fn del(&self, key:&str) -> bool{
       
        self.inner.remove(key).is_some()
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
            

            let key_to_remove:Vec<String> = self.inner.iter().filter_map(|entry|{
                if let Some(expire) = entry.expires_at{
                    if now >= expire{
                        return Some(entry.key().clone());
                    }
                }
                None
            })
            .collect();
        for key in key_to_remove {
            self.inner.remove(&key);
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