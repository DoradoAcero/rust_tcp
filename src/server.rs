use std::{io::Error, net::UdpSocket, thread::{self, sleep}, time::Duration};

use crate::{tcp_packet::{TcpPacket, MAX_PACKET_LENGTH}, unwrap_or_continue};

fn establish_connection(socket: &UdpSocket) -> Result<u32, Error> {
    // 1. recieve syn
    let mut buf = [0; MAX_PACKET_LENGTH];
    let (_, addr) = socket.recv_from(&mut buf)?;
    let syn_packet = TcpPacket::from_buffer(buf)?;
    if !syn_packet.flag_sync_seq_numbers {
        return Err(Error::new(std::io::ErrorKind::PermissionDenied, "Connection not open, Packet not syn"))
    }

    let syn_ack_pack = syn_packet.create_syn_ack();
    let seq_num = syn_ack_pack.sequence_number;
    let send_buf = syn_ack_pack.to_buffer();
    loop {
        // 2. send syn-ack
        socket.send_to(&send_buf, addr)?;

        // 3. recieve ack
        let mut buf = [0; MAX_PACKET_LENGTH];
        socket.recv_from(&mut buf)?;
        let ack_pack = TcpPacket::from_buffer(buf)?;
        if ack_pack.ack_number == seq_num + 1 {
            break;
        }
    };
    Ok(seq_num)
}

pub fn setup_server(server_addr: String){
    thread::spawn(move || -> Result<(), Error> {
        {
            
            let socket = UdpSocket::bind(server_addr)?;
            
            loop {
                let seq_num = unwrap_or_continue!(establish_connection(&socket));

                let mut buf = [0; MAX_PACKET_LENGTH];

                let mut messages: Vec<String> = vec![];
                let mut fin_flag = false;
                loop {
                    let (_, src) = socket.recv_from(&mut buf)?;
                    let packet = unwrap_or_continue!(TcpPacket::from_buffer(buf));
                    
                    let window_index = packet.sequence_number.wrapping_sub(seq_num);
                    messages.resize(window_index as usize, "".to_string());
                    messages.insert( window_index as usize, String::from_utf8(packet.data.clone()).unwrap());

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
                        // println!("{}: {}", message, messages.len());
                        break
                    };

                };
                sleep(Duration::new(0,500)); // implementing the final close wait
            } // the socket is closed here
        }
    });
}