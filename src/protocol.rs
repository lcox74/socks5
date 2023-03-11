use std::str::FromStr;


// Packet MAX Lengths
pub const CLIENT_IDENTIFIER_MAX_LENGTH: usize = 1 + 1 + 255;
pub const SERVER_SELECT_MAX_LENGTH: usize = 1 + 1;
pub const CLIENT_REQUEST_MAX_LENGTH: usize = 1 + 1 + 1 + 1 + (1 + 255) + 2;
pub const SERVER_RESPONSE_MAX_LENGTH: usize = 1 + 1 + 1 + 1 + (1 + 255) + 2;

// Version
pub const SOCKS5_VERSION: u8 = 5;

// Authentication
pub const NO_AUTHENTICATION: u8 = 0x00;
pub const NO_ACCEPTABLE_METHODS: u8 = 0xFF;

// Command
pub type Command = u8;
pub const CMD_ERROR: Command = 0x00;
pub const CMD_CONNECT: Command = 0x01;
pub const CMD_BIND: Command = 0x02;
pub const CMD_UDP_ASSOCIATE: Command = 0x03;

// Address Type
pub type AddressType = u8;
pub const ADDR_ERROR: AddressType = 0x00;
pub const ADDR_IPV4: AddressType = 0x01;
pub const ADDR_DOMAIN_NAME: AddressType = 0x03;
pub const ADDR_IPV6: AddressType = 0x04;

// Reply
pub type Reply = u8;
pub const REP_SUCCESS: Reply = 0x00;
pub const REP_GENERAL_FAILURE: Reply = 0x01;
pub const REP_CONNECTION_NOT_ALLOWED: Reply = 0x02;
pub const REP_NETWORK_UNREACHABLE: Reply = 0x03;
pub const REP_HOST_UNREACHABLE: Reply = 0x04;
pub const REP_CONNECTION_REFUSED: Reply = 0x05;
pub const REP_TTL_EXPIRED: Reply = 0x06;
pub const REP_COMMAND_NOT_SUPPORTED: Reply = 0x07;
pub const REP_ADDRESS_TYPE_NOT_SUPPORTED: Reply = 0x08;

#[derive(Clone)]
pub struct ClientIdentifier {
    version: u8,
    nmethods: u8,
    pub methods: Vec<u8>,
}

impl ClientIdentifier {
    pub fn from_bytes(data: [u8; CLIENT_IDENTIFIER_MAX_LENGTH], len: usize) -> ClientIdentifier {
        let mut methods = Vec::new();
        for i in 2..len {
            methods.push(data[i]);
        }

        ClientIdentifier {
            version: data[0],
            nmethods: data[1],
            methods: methods,
        }
    }

    pub fn verify(&self) -> bool {
        if self.version != 5 { return false; }
        if self.nmethods == 0 { return false; }
        if self.methods.len() != self.nmethods as usize { return false; }
        true
    }

    pub fn to_string(self) -> String {
        let mut s = String::new();
        for i in 0..self.methods.len() {
            s.push_str(&format!("{} ", self.methods[i]));
        }

        format!("ClientIdentifier: {{ VER: {}, NMETHODS: {}, METHODS: [ {}] }}", self.version, self.nmethods, s)
    }
}

#[derive(Clone)]
pub struct ServerSelect {
    version: u8,
    method: u8,
}

impl ServerSelect {
    pub fn new(method: u8) -> ServerSelect {
        ServerSelect {
            version: 0x05,
            method,
        }
    }

    pub fn to_bytes(self) -> ([u8; SERVER_SELECT_MAX_LENGTH], usize) {
        ([self.version, self.method], 2)
    }

    pub fn to_string(self) -> String {
        format!("ServerSelect: {{ VER: {}, METHOD: {} }}", self.version, self.method)
    }
}

#[derive(Clone)]
pub struct ClientRequest {
    version: u8,
    pub command: Command,
    rsv: u8,
    address_type: AddressType,
    pub address: String,
    port: u16,
}

impl ClientRequest {
    pub fn from_bytes(data: [u8; CLIENT_REQUEST_MAX_LENGTH], _len: usize) -> ClientRequest {

        let mut req = ClientRequest {
            version: data[0],
            command: data[1],
            rsv: data[2],
            address_type: data[3],
            address: String::new(),
            port: 0,
        };

        match data[3] {
            ADDR_IPV4 => {
                // IPv4
                let x: [u8; 4] = data[4..8].try_into().expect("slice with incorrect length");
                req.address = std::net::Ipv4Addr::from(x).to_string();
                req.port = u16::from_be_bytes(data[8..10].try_into().expect("slice with incorrect length"));
            },
            ADDR_DOMAIN_NAME => {
                // Domain Name
                let mut s = String::new();
                for i in 5..(5 + data[4]) as usize {
                    s.push(data[i] as char);
                }
                req.address = s;

                let port_start = 5 + data[4] as usize;
                req.port = u16::from_be_bytes(data[port_start..(port_start + 2)].try_into().expect("slice with incorrect length"));
            },
            ADDR_IPV6 => {
                // IPv6
                let x: [u8; 16] = data[4..20].try_into().expect("slice with incorrect length");
                req.address = std::net::Ipv6Addr::from(x).to_string();
                req.port = u16::from_be_bytes(data[20..22].try_into().expect("slice with incorrect length"));
            },
            _ => {
                // Error
            }

        }
        req
    }

    pub fn get_addr(self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn verify(&self) -> bool {
        if self.version != 5 { return false; }
        if ![CMD_CONNECT, CMD_BIND, CMD_UDP_ASSOCIATE].contains(&self.command) { return false; }
        if self.rsv != 0 { return false; }
        if ![ADDR_IPV4, ADDR_DOMAIN_NAME, ADDR_IPV6].contains(&self.address_type) { return false; }
        if self.port == 0 { return false; }
        true
    }

    pub fn to_string(self) -> String {
        format!("ClientRequest: {{ VER: {}, CMD: {}, RSV: {}, ATYP: {}, DST.ADDR: {}, DST.PORT: {} }}", self.version, self.command, self.rsv, self.address_type, self.address, self.port)
    }

}

#[derive(Clone)]
pub struct ServerResponse {
    version: u8,
    reply: Reply,
    rsv: u8,
    address_type: AddressType,
    pub address: String,
    port: u16,
}

impl ServerResponse {

    pub fn new(reply: u8, address_type: AddressType, address: String, port: u16) -> ServerResponse {
        ServerResponse {
            version: 0x05,
            reply,
            rsv: 0x00,
            address_type,
            address,
            port,
        }
    }

    pub fn rep_succeeded() -> ServerResponse { ServerResponse::new(REP_SUCCESS, ADDR_IPV4, "0.0.0.0".to_string(), 0) }

    pub fn to_bytes(self) -> ([u8; SERVER_RESPONSE_MAX_LENGTH], usize) {
        let mut data = [0u8; SERVER_RESPONSE_MAX_LENGTH];
        data[0] = self.version;
        data[1] = self.reply;
        data[2] = self.rsv;
        data[3] = self.address_type;

        let mut size = 0;

        match data[3] {
            ADDR_IPV4 => {
                // IPv4
                let addr = std::net::Ipv4Addr::from_str(self.address.as_str()).unwrap().octets();
                for i in 0..4 {
                    data[4 + i] = addr[i];
                }

                // Port
                data[8] = (self.port >> 8) as u8;
                data[9] = self.port as u8;

                size = 10;
            },
            ADDR_DOMAIN_NAME => {
                // Domain Name
                let addr: Vec<u8> = self.address.as_bytes().to_vec();
                data[4] = addr.len() as u8;
                for i in 0..addr.len() {
                    data[5 + i] = addr[i];
                }

                // Port
                data[5 + addr.len()] = (self.port >> 8) as u8;
                data[6 + addr.len()] = self.port as u8;

                size = 7 + addr.len();
            },
            ADDR_IPV6 => {
                // IPv6
                let addr = std::net::Ipv6Addr::from_str(self.address.as_str()).unwrap().octets();
                for i in 0..16 {
                    data[4 + i] = addr[i];
                }
            
                // Port 
                data[20] = (self.port >> 8) as u8;
                data[21] = self.port as u8;

                size = 22;
            },
            _ => {
                // Error
            }
        }
        (data, size)
    }

    pub fn to_string(self) -> String {
        format!("ServerResponse: {{ VER: {}, REP: {}, RSV: {}, ATYP: {}, BND.ADDR: {}, BND.PORT: {} }}", self.version, self.reply, self.rsv, self.address_type, self.address, self.port)
    }
}