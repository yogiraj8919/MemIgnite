// Command module for mini_redis
#[derive(Debug)]
pub enum Command {
    Ping,
    Echo(String),
    Set {key:String, value:String,ex:Option<u64>,exat:Option<u64>},
    Get {key:String},
    Del {key:String},
    Help,
    Quit,
    Unknown(String)
}