use std::net::TcpStream;
use std::time::Duration;
use std::thread;
use std::sync::mpsc;
use std::path::Path;

use crate::repl;
use crate::flag;
use crate::setting;

use getch::Getch;

pub fn run(host:      &str,
           mut flags: flag::Flags,
           params:    Option<setting::Params>)
{
    let receiver = TcpStream::connect_timeout(&to_SocketAddr(host), Duration::from_secs(4))
        .unwrap_or_else(|e| {
            eprintln!("Could not connect: {}", e);
            std::process::exit(1);
        });
    receiver.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    let transmitter = receiver.try_clone().expect("Failed to clone from receiver");

    let (tx, rx) = mpsc::channel();

    // If write_file is already exists
    if let Some(write_file) = flags.write_file() {
        if Path::new(write_file).exists() {
            if !flags.is_append() {
                let g = Getch::new();
                println!("\"{}\" is already exists!", write_file);
                println!("Press ENTER to continue overwrite");
                match g.getch() {
                    Ok(b'\n') | Ok(b'\r') => (),  // continue
                    _ => std::process::exit(0),   // exit
                }
            }
        } else if flags.is_append() {
            let g = Getch::new();
            println!("\"{}\" is not exists!", write_file);
            println!("Press ENTER to create the file and continue");
            match g.getch() {
                Ok(b'\n') | Ok(b'\r') => (),  // continue
                _ => std::process::exit(0),   // exit
            }
            flags.set_append(false);
        }
    }

    // Check if params exists
    if params.is_none() {
        flags.set_nocolor(true);
    }

    println!("Connected. {}:", host);
    println!("Type \"~.\" to exit.");

    // Receiver
    let handle = thread::spawn(move || {
        repl::receiver_run(receiver, rx, flags, params);
    });

    // Transmitter
    repl::transmitter_run(transmitter, tx);

    handle.join().unwrap();
}

// Check if the port number is attached
// If not attached, append ":23"
#[allow(non_snake_case)]
fn to_SocketAddr(host: &str) -> std::net::SocketAddr {
    match host.parse() {
        Ok(result) => result,
        Err(_) => {
            let mut host = host.to_string();
            host.push_str(":23");
            host.parse::<std::net::SocketAddr>().unwrap()
        }
    }
}