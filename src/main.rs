use std::{net::{TcpListener, TcpStream}, io::{Read, Write, self}, sync::Arc};

pub mod protocol;

fn main() {
    println!("Starting SOCKS5 proxy server...");
    let listener = TcpListener::bind("0.0.0.0:1080").expect("Failed to bind to port 1080");

    // Listen for incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { std::thread::spawn(move || handle_connection(stream)); },
            Err(e) => println!("Error: {}", e)
        }
    }
}


fn handle_connection(stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());

    // Step 1: Initial Connection
    match initial_connection(&stream) {
        Ok(_) => println!("Initial connection established"),
        Err(e) => { println!("Error: {}", e); return; }
    }

    // Step 2: SOCKS Request
    match socks_request(&stream) {
        Ok(remote_stream) => {
            // Step 3: Relay data between client and remote server
            relay_streams(&stream, &remote_stream)
        },
        Err(e) => println!("Error: {}", e)
    }

    println!("Connection closed");
}

/// Relay data between the client and the remote server
fn relay_streams(client_stream: &TcpStream, remote_stream: &TcpStream) {
   // Create Arcs for the streams
   let client_conn = Arc::new(client_stream);
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
}

/// Initialise the connection with the client. Confirm correct SOCKS version
/// and select authentication method.
fn initial_connection(mut stream: &TcpStream) -> Result<(), &'static str> {
    // Set up buffer
    let mut buffer = [0 as u8; 1024];
    
    // Default Authentication Method
    let mut selected_method = protocol::NO_ACCEPTABLE_METHODS;

    // Identify Client Version and Authentication Methods
    match stream.read(&mut buffer) {
        Ok(len) => {
            let p1 = protocol::ClientIdentifier::from_bytes(&buffer, len);
            if !p1.verify() { return Err("invalid client identifier"); }

            // Select Authentication Method
            if p1.methods.contains(&protocol::NO_AUTHENTICATION) {
                selected_method = protocol::NO_AUTHENTICATION;
            }
        },
        Err(_) => { return Err("error reading from stream"); }
    }
    
    // Send Authentication Method
    let p2 = protocol::ServerSelect::new(selected_method);
    let (d, len) = p2.to_bytes();
    stream.write(&d[..len]).unwrap();
    
    // Check if there was a valid authentication method
    if selected_method == protocol::NO_ACCEPTABLE_METHODS { return Err("no acceptable authentication methods"); }

    // No further authentication required

    Ok(())
}

/// Handle SOCKS Requests from the client, will will be after the initial 
/// connection has been established and authentication has been completed.
fn socks_request(mut stream: &TcpStream) -> Result<TcpStream, &'static str> {
    // Set up buffer
    let mut buffer = [0; 1024];

    // Read SOCKS Request
    match stream.read(&mut buffer) {
        Ok(len) => {
            // Parse request
            let req = protocol::ClientRequest::from_bytes(&buffer, len);
            if !req.verify() { return Err("SOCKS request incorrect"); }

            // Handle request
            match req.command {
                protocol::CMD_CONNECT => {
                    match TcpStream::connect(req.get_addr()) {
                        Ok(remote_stream) => {
                            // Send success response
                            let p3 = protocol::ServerResponse::rep_succeeded();
                            let (d, len) = p3.to_bytes();
                            stream.write(&d[..len]).unwrap();

                            return Ok(remote_stream);
                        },
                        Err(_e) => { return Err("Error connecting to remote server"); }
                    }
                },
                protocol::CMD_BIND => { /* TODO: Bind command implementation */ },
                protocol::CMD_UDP_ASSOCIATE => { /* TODO: UDP Associate command implementation */ },
                _ => { return Err("Invalid command"); } // Verify should have caught this
            }

            return Err("command not implemented");

        },
        Err(_e) => { return Err("Error reading from stream"); }
    }
}