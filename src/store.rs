use std::{ cmp::Ordering, collections::BinaryHeap, sync::{Arc, Mutex}, time::{Duration, SystemTime, UNIX_EPOCH}};

use tokio::time::interval;


use dashmap::{DashMap};



#[derive(Clone)]
pub struct Store{
    inner:Arc<DashMap<String,Entry>>,
    expirations:Arc<Mutex<BinaryHeap<EntryItem>>>
}



struct Entry{
    value:String,
    expires_at:Option<u64>
}


#[derive(Debug)]
struct EntryItem{
    expires_at: u64,
    key:String
}

impl Ord for EntryItem{
    fn cmp(&self,other:&Self) -> Ordering{
        other.expires_at.cmp(&self.expires_at)
    }
}

impl PartialOrd for EntryItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for EntryItem {
    fn eq(&self, other: &Self) -> bool {
        self.expires_at == other.expires_at
    }
}

impl Eq for EntryItem {}


impl Store {
     pub fn new() -> Self{
        Store{
            inner : Arc::new(DashMap::new()),
            expirations:Arc::new(Mutex::new(BinaryHeap::new()))
        }
    }

    pub async fn set(&self,key:String,value:String,ttl:Option<Duration>){
    

        let expires_at = ttl.map(|d| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + d.as_secs()
        });
     
        self.inner.insert(key.clone(),Entry { value, expires_at });
        if let Some(expire) = expires_at {
            let mut heap = self.expirations.lock().unwrap();
            heap.push(EntryItem { expires_at:expire, key });
        }
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
                    // Remove from heap
                    let mut heap = self.expirations.lock().unwrap();
                    heap.retain(|item| item.key != key);
                    return None;
                }
               
            }
  

            return Some(entry.value.clone());
        }
        None
    }

    pub async fn del(&self, key:&str) -> bool{
       
        let removed = self.inner.remove(key).is_some();
        if removed {
            let mut heap = self.expirations.lock().unwrap();
            heap.retain(|item| item.key != key);
        }
        removed
    }

    pub fn start_expiration_task(self){
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));
            loop{
                ticker.tick().await;
                let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            

                loop{
                    let mut heap = self.expirations.lock().unwrap();
                    let item = match heap.peek(){
                        Some(item) if item.expires_at <= now => heap.pop().unwrap(),
                        _ => break
                    };
                    drop(heap);

                    if let Some(entry) = self.inner.get(&item.key){
                        if entry.expires_at == Some(item.expires_at){
                            drop(entry);
                            self.inner.remove(&item.key);
                        }
                    } 
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