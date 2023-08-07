use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::{Read, Write};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets config file for logging (log4rs)
    #[arg(short, long, value_name = "FILE")]
    sockaddr: Option<String>,
}

fn stream_handler(stream: &mut TcpStream) {
    let mut buff = [0u8; 128];

    loop {
        match stream.read(&mut buff) {
            Ok(_) => {}
            Err(_) => {
                println!("Error when read");
                break;
            }
        }

        match stream.write(&buff) {
            Ok(_) => {}
            Err(_) => {
                println!("Error when write");
                break;
            }
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let sockaddr = match &args.sockaddr {
        Some(_sockaddr) => _sockaddr.as_str(),
        None => "127.0.0.1:8080",
    };

    let listener = TcpListener::bind(sockaddr).unwrap();
    for stream in listener.incoming() {
        stream_handler(&mut stream?);
    }

    Ok(())
}
