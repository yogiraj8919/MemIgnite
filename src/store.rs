use std::{cmp::Ordering, collections::{BinaryHeap, VecDeque}, sync::{Arc}, time::{Duration, SystemTime, UNIX_EPOCH}};

use tokio::time::interval;

use tokio::sync::Mutex;

use tokio::sync::mpsc;

use dashmap::{DashMap};

use crate::aof::Aof;

#[allow(unused)]

#[derive(Clone)]
pub struct Store{
    pub inner:Arc<DashMap<String,Entry>>,
    expirations:Arc<Mutex<BinaryHeap<EntryItem>>>,
    pub aof: Arc<Mutex<Aof>>,
    pub aof_tx: mpsc::UnboundedSender<String>,
    pub max_keys:usize
}

#[derive(Clone)]
pub enum Value{
     String(String),
    List(VecDeque<String>)
}

#[derive(Clone)]
pub struct Entry{
    pub value:Value,
    pub expires_at:Option<u64>,
    pub last_acces: u64
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
     pub fn new(aof:Arc<Mutex<Aof>>, aof_tx: mpsc::UnboundedSender<String>) -> Self{
        Store{
            inner : Arc::new(DashMap::new()),
            expirations:Arc::new(Mutex::new(BinaryHeap::new())),
            aof,
            aof_tx,
            max_keys:3
        }
    }

    pub async fn set(&self,key:String,value:String,ttl:Option<Duration>){
    

        self.set_internal(key.clone(), value.clone(), ttl).await;

        

        if let Some(expire) = ttl{
            let expire_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + expire.as_secs();
            self.aof_tx.send(format!("SET {} {} EXAT {}",key,value,expire_ts)).ok();
        }else{
            self.aof_tx.send(format!("SET {} {}",key,value)).ok();
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
    }


    async fn set_internal(&self, key:String, value:String, ttl:Option<Duration>){
        self.evict_lru_if_needed();
        let expires_at = ttl.map(|d|{
            SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + d.as_secs()
        });

        self.inner.insert(key.clone(), Entry { value: Value::String(value), expires_at,last_acces:Self::now_secs() });
        
        if let Some(expire) = expires_at {
            let mut heap = self.expirations.lock().await;
            heap.push(EntryItem { expires_at:expire, key });
        }
    }

    pub async fn lpush(&self,key:String, value:String) -> usize{
      let len = if let Some(mut entry) = self.inner.get_mut(&key){
        match &mut entry.value{
            Value::List(list)=>{
                list.push_front(value.clone());
                list.len()
            }
            _ =>  0
        }
      }else{
        let mut new_list = VecDeque::new();
        new_list.push_front(value.clone());
        self.inner.insert(key.clone(), Entry { value: Value::List(new_list), expires_at: None,last_acces:Self::now_secs() }
        );
         1
      };

      
      self.aof_tx.send(format!("LPUSH {} {}",key,value)).ok();

      len

    }

    pub async fn rdrop(&self,key:&str) -> Option<String>{
        let result = if let Some(mut entry) = self.inner.get_mut(key){
            match &mut entry.value {
                Value::List(list) => list.pop_back(),
               _ =>  None
            }
        }else{
            None
        };

        if result.is_some(){
            
            self.aof_tx.send(format!("RDROP {}",key)).ok();
        }

        result
        
    }

    pub async fn get(&self,key:&str) -> Option<String>{

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
           .as_secs();
        
        if let Some(mut entry) = self.inner.get_mut(key){
            if let Some(expire) = entry.expires_at{
                if now >= expire{
                    drop(entry); 
                    //expired -> delete
                    self.inner.remove(key);
                    // Remove from heap
                    let mut heap = self.expirations.lock().await;
                    heap.retain(|item| item.key != key);
                    return None;
                }
               
            }

            entry.last_acces = Self::now_secs();

            match &entry.value{
                Value::String(s) => {
                    return Some(s.clone());
                }
                Value::List(_)=>return None
            }
        }
        None
    }

    pub async fn del(&self, key:&str) -> bool{
       
        let removed = self.inner.remove(key).is_some();
        if removed {
            let mut heap = self.expirations.lock().await;
            heap.retain(|item| item.key != key);

            
            self.aof_tx.send(format!("DEL {}",key)).ok();
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
                    let mut heap = self.expirations.lock().await;
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
                   self.set_internal(key, value, Some(Duration::from_secs(remaining)))
                   .await;
               }else {
                   self.set_internal(key, value, None).await;
               }
               

            }
            Command::Del { key }=> {
                self.del(&key).await;
            }
            _ =>{}
        }

    }

    pub fn snapshot(&self) -> Vec<(String, Entry)> {
        self.inner
        .iter()
        .map(|item| (item.key().clone(), item.value().clone()))
        .collect()
    }

    pub fn evict_lru_if_needed(&self){
        if self.inner.len() < self.max_keys{
            return;
        }
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = u64::MAX;

        for item in self.inner.iter(){
            let access_time = item.value().last_acces;

            if access_time < oldest_time{
                oldest_time = access_time;
                oldest_key = Some(item.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            self.inner.remove(&key);
        }
    }

}