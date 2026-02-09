use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::protocol::handle_commands;
use crate::protocol::parsing::RedisProtocol;

mod commands;
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
                        handle_commands(redis_data, &mut write_buf);
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
    send_response(stream, &mut format!("-ERR {}\r\n", error));
}
