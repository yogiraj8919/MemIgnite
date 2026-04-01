use std::sync::{
    Arc,
    atomic::{AtomicU64,Ordering}
};

use std::time::{SystemTime,UNIX_EPOCH};

#[derive(Clone)]
pub struct Stats{
    pub commands_processed: Arc<AtomicU64>,
    pub clients_connected: Arc<AtomicU64>,
    pub rewrite_count: Arc<AtomicU64>,
    pub start_time: u64
}

impl Stats {
    pub fn new() -> Self{
        let start_time = SystemTime::now().
        duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
        
        Self { commands_processed: Arc::new(AtomicU64::new(0)), clients_connected: Arc::new(AtomicU64::new(0)), rewrite_count: Arc::new(AtomicU64::new(0)), start_time }
    }

    pub fn incr_commands(&self){
        self.commands_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_clients(&self){
        self.clients_connected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn incr_rewrite(&self){
        self.rewrite_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn uptime(&self) -> u64{
        let now = SystemTime::now().
        duration_since(UNIX_EPOCH).unwrap().as_secs();

        now - self.start_time
    }
}