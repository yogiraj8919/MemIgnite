use crate::command::Command;

pub fn parse_command(input:&str) -> Command{
    let input = input.trim();
    if input.is_empty(){
        return Command::Unknown("empty".to_string());
    }

    let mut parts  = input.split_whitespace();

    let cmd = parts.next().unwrap().to_uppercase();
    match cmd.as_str() {
        "PING" => Command::Ping,
        "HELP" => Command::Help,
        "SET" => {
            let key = match parts.next() {
                Some(k) => k.to_string(),
                None => return Command::Unknown("SET missing key".to_string()),
            };
            let value = match parts.next() {
                Some(v) => v.to_string(),
                None => return Command::Unknown("SET missing value".to_string()),
            };

            let mut ex = None;
            let mut exat = None;

            while let Some(flag) = parts.next() {
                if flag.eq_ignore_ascii_case("EX") {
                    if let Some(sec) = parts.next() {
                        match sec.parse::<u64>() {
                            Ok(n)if n > 0 => ex = Some(n),
                            Ok(_) => return Command::Unknown("EX must be > 0".into()),
                            Err(_) => return Command::Unknown("Invalid EX value".into())

                        }
                    }
                }else if flag.eq_ignore_ascii_case("EXAT") {
                        if let Some(ts) = parts.next() {
                            match ts.parse::<u64>(){
                                Ok(n) if n > 0 => exat = Some(n),
                                Ok(_) => return Command::Unknown("EXAT must be > 0".into()),
                                Err(_) => return Command::Unknown("Invalid EXAT value".into()) 
                            }
                            
                        }
                }
            }

            Command::Set { key, value ,ex,exat}
        },
        "GET" => {
            let key:String = match parts.next() {
                Some(k) => k.to_string(),
                None => return Command::Unknown("GET missing key".to_string())
            };
            Command::Get { key }
        }
        "DEL" => {
            let key:String = match parts.next() {
                Some(k) => k.to_string(),
                None => return Command::Unknown("DEL missing key".to_string())
            };
            Command::Del { key } 
        }
        "LPUSH" => {
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();
            Command::LPUSH { key, value }
        }
        "RDROP" =>{
            let key = parts.next().unwrap().to_string();
            Command::RDROP { key }
        }
        "ECHO" => Command::Echo(parts.collect::<Vec<_>>().join(" ")),
        "QUIT" => Command::Quit,
        other=> Command::Unknown(other.to_string()),
    }
}