use anyhow::Result;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::protocol::handle_actions;
use crate::protocol::parsing::RedisProtocol;

mod protocol;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_stream(stream);
                });
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        };
    }
    Ok(())
}

fn handle_stream(mut stream: TcpStream) {
    let mut stream_buf = String::new();
    let mut write_buf = String::new();
    match stream.read_to_string(&mut stream_buf) {
        Ok(nbytes) => {
            if nbytes == 0 {
                return;
            }
            println!("{}", &stream_buf);
            match RedisProtocol::from_str(&stream_buf) {
                Ok((_, redis_data)) => {
                    if !redis_data.valid() {
                        eprintln!(
                            "Data sent over doesn't match redis protocol - parameters length specified doesn't equal number of parameters sent",
                        );
                    }
                    handle_actions(redis_data, &mut write_buf);
                }
                Err(e) => {
                    eprintln!("Failed to parse the incoming stream: {}", e);
                }
            }

            println!("{}", write_buf);
            println!("{:?}", write_buf.as_bytes());
            if let Err(e) = stream.write_all(write_buf.as_bytes()) {
                eprintln!("Failed to write response: {}", e);
            }
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
