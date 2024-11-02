use std::{net::UdpSocket, thread, time::Duration};

use tcp_packet::{string_to_packets, TcpPacket, MAX_PACKET_LENGTH};
mod tcp_packet;

fn main() -> std::io::Result<()>{
    {
        let server_addr ="127.0.0.1:8000".to_string();
        let client_addr = "127.0.0.1:8001".to_string();
        setup_server(server_addr.clone());
        // wait for server, 0.5s
        thread::sleep(Duration::new(0, 500000000));

        let socket = UdpSocket::bind(client_addr).expect("couldn't bind to client address");
        let packets = string_to_packets("test".to_string());
        for packet in packets {
            let buf = packet.to_buffer();
            socket.send_to(&buf, server_addr.clone())?;
        }
        let mut buf = [0; MAX_PACKET_LENGTH];
        socket.recv_from(&mut buf)?;
        println!("{:?}", TcpPacket::from_buffer(buf));
    }
    Ok(())
}

fn setup_server(server_addr: String){
    thread::spawn(move || -> std::io::Result<()> {
            {
                let socket = UdpSocket::bind(server_addr)?;

                // Receives a single datagram message on the socket. If `buf` is too small to hold
                // the message, it will be cut off.
                let mut buf = [0; MAX_PACKET_LENGTH];
                let (_, src) = socket.recv_from(&mut buf)?;

                // Redeclare `buf` as slice of the received data and send reverse data back to origin.
                let packet = TcpPacket::from_buffer(buf);
                println!("{:?}", packet);
                let ack_pack = packet.create_ack();
                socket.send_to(&ack_pack.to_buffer(), &src)?;
            } // the socket is closed here
            Ok(())
        }
    );
}