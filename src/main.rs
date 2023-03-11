use std::{net::{TcpListener, TcpStream}, io::{Read, Write, self}, sync::Arc};

pub mod protocol;

fn main() {
    println!("Starting SOCKS5 proxy server...");
    let listener = TcpListener::bind("0.0.0.0:1080").unwrap();

    // Listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }

}


fn handle_connection(stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());

    if !initial_connection(&stream) { return; }
    match socks_request(&stream) {
        Ok(remote_stream) => {
            // Create Arcs for the streams
            let client_conn = Arc::new(stream);
            let remote_conn = Arc::new(remote_stream);

            // Get client and remote streams
            let (mut client_tx, mut client_rx) = (client_conn.try_clone().unwrap(), client_conn.try_clone().unwrap());
            let (mut remote_tx, mut remote_rx) = (remote_conn.try_clone().unwrap(), remote_conn.try_clone().unwrap());

            // Create threads to handle data transfer
            let client_to_remote = std::thread::spawn(move || io::copy(&mut client_tx, &mut remote_rx).unwrap());
            let remote_to_client = std::thread::spawn(move || io::copy(&mut remote_tx, &mut client_rx).unwrap());

            // Wait for threads to finish
            client_to_remote.join().unwrap();
            remote_to_client.join().unwrap();
        },
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
        
    }

    println!("Connection closed");
}

/// Initialise the connection with the client. Confirm correct SOCKS version
/// and select authentication method.
fn initial_connection(mut stream: &TcpStream) -> bool {
    
    // Identify Client Version and Authentication Methods
    let mut buffer = [0; protocol::CLIENT_IDENTIFIER_MAX_LENGTH];
    let mut selected_method = protocol::NO_ACCEPTABLE_METHODS; // Default
    match stream.read(&mut buffer) {
        Ok(len) => {
            let p1 = protocol::ClientIdentifier::from_bytes(buffer, len);
            println!("{}", p1.clone().to_string());
            if !p1.verify() {
                println!("Client version or authentication method incorrect");
                return false;
            }

            // Select Authentication Method
            if p1.methods.contains(&protocol::NO_AUTHENTICATION) {
                selected_method = protocol::NO_AUTHENTICATION;
            }
        },
        Err(e) => {
            println!("Error reading from stream: {}", e);
            return false;
        }
    }
    
    // Send Authentication Method
    let p2 = protocol::ServerSelect::new(selected_method);
    println!("{}", p2.clone().to_string());
    let (d, len) = p2.to_bytes();
    stream.write(&d[..len]).unwrap();
    
    if selected_method == protocol::NO_ACCEPTABLE_METHODS {
        println!("No acceptable authentication methods");
        return false;
    }

    // No further authentication required

    true
}

/// Handle SOCKS Requests from the client, will will be after the initial 
/// connection has been established and authentication has been completed.
fn socks_request(mut stream: &TcpStream) -> Result<TcpStream, &'static str> {

    // Read SOCKS Request
    let mut buffer = [0; protocol::CLIENT_REQUEST_MAX_LENGTH];
    match stream.read(&mut buffer) {
        Ok(len) => {
            let req = protocol::ClientRequest::from_bytes(buffer, len);
            println!("{}", req.clone().to_string());
            if !req.verify() {
                return Err("SOCKS request incorrect");
            }

            match req.command {
                protocol::CMD_CONNECT => {
                    match TcpStream::connect(req.get_addr()) {
                        Ok(remote_stream) => {
                            // Send success response
                            let p3 = protocol::ServerResponse::rep_succeeded();
                            println!("{}", p3.clone().to_string());
                            let (d, len) = p3.to_bytes();
                            stream.write(&d[..len]).unwrap();

                            return Ok(remote_stream);
                        },
                        Err(_e) => {
                            return Err("Error connecting to remote server");
                        }
                    }
                },
                protocol::CMD_BIND => {
                    // TODO: Bind command implementation
                    
                },
                protocol::CMD_UDP_ASSOCIATE => {
                    // TODO: UDP Associate command implementation
                },
                _ => { return Err("Invalid command"); } // Verify should have caught this
            }

            return Err("command not implemented");

        },
        Err(_e) => {
            return Err("Error reading from stream");
        }
    }
}