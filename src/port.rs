use std::{io::Error, net::UdpSocket};

use crate::{client::send_message, server::recieve_message};



pub struct TcpPort {
    udp_port: UdpSocket,
}

impl TcpPort {
    pub fn new(addr: &String) -> Result<TcpPort, Error> {
        let port = UdpSocket::bind(addr)?;
        Ok(TcpPort { udp_port: port })
    }


    pub fn send(&self, message: String, addr: &String) -> Result<(), Error> {
        send_message(message, &self.udp_port, addr)?;
        Ok(())
    }

    pub fn recieve(&self) -> Result<String, Error> {
        recieve_message(&self.udp_port)
    }
}