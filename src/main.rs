use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::commands::handle_commands;
use crate::database::cache::RedisCache;
use crate::protocol::parsing::RedisProtocol;
use crate::server::replica::ReplicaInfo;
use crate::server::server::RedisServer;

mod commands;
mod database;
mod protocol;
mod server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    port: Option<String>,

    #[arg(long)]
    replicaof: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let port = match args.port {
        Some(port) => port,
        None => "6379".to_string(),
    };

    let role;
    let replicaof = match args.replicaof {
        Some(replicaof) => {
            role = "slave".to_string();
            Some(ReplicaInfo::arg_parse(replicaof)?)
        }
        None => {
            role = "master".to_string();
            None
        }
    };

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    let cache = Arc::new(Mutex::new(HashMap::new()));
    let server = Arc::new(RedisServer::new(role, replicaof));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cache_clone = Arc::clone(&cache);
                let server_clone = Arc::clone(&server);
                thread::spawn(|| {
                    handle_stream(stream, cache_clone, server_clone);
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        };
    }
    Ok(())
}

fn handle_stream(mut stream: TcpStream, cache: RedisCache, server: Arc<RedisServer>) {
    loop {
        let mut reader = BufReader::new(&mut stream);
        let mut stream_buf = String::new();
        let mut write_buf = String::new();
        match reader.read_line(&mut stream_buf) {
            Ok(nbytes) => {
                if nbytes == 0 {
                    return;
                }

                // Check if first value is a *
                if &stream_buf[0..1] != "*" {
                    send_error(&mut stream, "First value is not *");
                    continue;
                }

                // Grab rest of lines based on first number
                if let Ok(param_n) = &stream_buf[1..].trim().parse::<usize>() {
                    for _ in 0..param_n * 2 {
                        match reader.read_line(&mut stream_buf) {
                            Ok(_) => {}
                            Err(e) => write_buf.push_str(&format!(
                                "Failed to read lines that should be there: {}",
                                e
                            )),
                        }
                    }
                } else {
                    write_buf.push_str("Could not parse the number of parameters sent via protocol")
                }
                // Anything written in is an error at this point
                if !write_buf.is_empty() {
                    send_error(&mut stream, &write_buf);
                    continue;
                }

                match RedisProtocol::from_str(&stream_buf) {
                    Ok((_, redis_data)) => {
                        if !redis_data.valid() {
                            send_error(
                                &mut stream,
                                "Data sent over doesn't match redis protocol - parameters length specified doesn't equal number of parameters sent",
                            );
                            continue;
                        }
                        handle_commands(redis_data, &mut write_buf, &cache, &server);
                    }
                    Err(e) => {
                        send_error(
                            &mut stream,
                            &format!("Failed to parse the incoming stream: {}", e),
                        );
                        continue;
                    }
                }
                send_response(&mut stream, &mut write_buf);
            }
            Err(e) => send_error(&mut stream, &format!("Failed to read from stream: {}", e)),
        }
    }
}

fn send_response(stream: &mut TcpStream, write_buf: &mut String) {
    if let Err(e) = stream.write_all(write_buf.as_bytes()) {
        eprintln!("Failed to write response: {}", e);
    }
    if let Err(e) = stream.flush() {
        eprintln!("Failed to flush stream: {}", e);
    }
}

fn send_error(stream: &mut TcpStream, error: &str) {
    eprintln!("{}", error);
    send_response(stream, &mut format!("- {}\r\n", error));
}
