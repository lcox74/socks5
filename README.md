# Socks v5
> 11th March 2023

This is a small side project aimed at implementing a Socks version 5 server 
according to the [RFC1928] specification using the Rust programming language. 
The goal of this project is to learn more about networking protocols and 
the Rust language.

## Features

The implemented Socks version 5 server supports the following features:

- **Authentication methods:** No Authentication Required (`0x00`) 
- **Command types:** Connect (`0x01`)
- **Address types:** IPv4 (`0x01`), IPv6 (`0x04`), and Domain Name (`0x03`).

## Usage

To use the Socks version 5 server, configure your client to use the server as a proxy. The client should connect to the 
server using the SOCKS protocol on port `1080` (or the port you specified).

When the client sends a request to the server, the server will respond according to the [RFC1928] specification. The 
server will then forward the client's request to the destination server and relay the response back to the client.

To test the program locally you can run it using `cargo run` then use curl to proxy a request through it, below is an
example of how to use curl:

```bash
# Make a request to the SOCKS v5 RFC page while proxying 
# it through the SOCKS server
curl -x socks5://localhost:1080 https://www.rfc-editor.org/rfc/rfc1928
```

[RFC1928]: https://www.rfc-editor.org/rfc/rfc1928