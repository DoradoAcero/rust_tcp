use std::{fs, io::Result, net::UdpSocket, thread, time::Duration};

use client::send_message;
use tcp_packet::{TcpPacket, MAX_PACKET_LENGTH};
mod tcp_packet;
mod client;
mod server;
mod utils;


fn main() -> Result<()>{
    {
        let server_addr ="127.0.0.1:8000".to_string();
        let client_addr = "127.0.0.1:8001".to_string();
        setup_server(server_addr.clone());
        // wait for server, 0.5s
        thread::sleep(Duration::new(0, 500000000));


        let contents = fs::read_to_string("bee_movie.txt")
            .expect("Should have been able to read the file");

        let socket = UdpSocket::bind(client_addr).expect("couldn't bind to client address");
        send_message(contents, socket, &server_addr)?
    }
    Ok(())
}

fn setup_server(server_addr: String){
    thread::spawn(move || -> Result<()> {
            {
                let socket = UdpSocket::bind(server_addr)?;

                let mut buf = [0; MAX_PACKET_LENGTH];

                let mut messages: Vec<String> = vec![];
                let mut fin_flag = false;
                loop {
                    let (_, src) = socket.recv_from(&mut buf)?;
                    let packet = unwrap_or_continue!(TcpPacket::from_buffer(buf));
                    
                    messages.resize(packet.sequence_number as usize, "".to_string());
                    messages.insert( packet.sequence_number as usize, String::from_utf8(packet.data.clone()).unwrap());

                    if packet.flag_finished {
                        fin_flag = true;
                    }

                    let ack_pack = packet.create_ack();
                    socket.send_to(&ack_pack.to_buffer(), &src)?;

                    // if there are no empty packets, and the last message has been recieved
                    if fin_flag && !messages.contains(&"".to_string()){
                        let mut message = String::new();
                        for i in 0..messages.len() {
                            message.push_str(&messages[i].clone());
                        }
                        println!("{}", message);
                        
                        fin_flag = false;
                    };

                };
            } // the socket is closed here
        }
    );
}