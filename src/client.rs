use std::{cmp::min, io::{Error, ErrorKind, Result}, net::UdpSocket, time::{Duration, Instant}};
use rand::Rng;

use crate::tcp_packet::{create_syn_packet, string_to_packets, TcpPacket, MAX_PACKET_LENGTH, TCP_WINDOW_LENGTH};

enum PacketStatus {
    Unsent,
    Acknowledged,
    Sent{retry_time: Instant, retry_count: u32},
}

fn get_retry_time(retry_count: u32) -> Instant {
    Instant::now() + Duration::new(retry_count as u64, rand::thread_rng().gen_range(0..1000000000))
}

fn establish_connection(socket: &UdpSocket, addr: &str) -> Result<u32> {
    // 1. send syn
    let syn_packet = create_syn_packet();
    let syn_num = syn_packet.sequence_number;
    let send_buf = syn_packet.to_buffer();

    // keep sending until we reach max retries
    let syn_ack_pack: TcpPacket = loop {
        socket.send_to(&send_buf, addr)?;

        // 2. recieve syn-ack
        let mut buf = [0; MAX_PACKET_LENGTH];
        socket.recv_from(&mut buf);
        let syn_ack_pack = TcpPacket::from_buffer(buf)?;
        if syn_ack_pack.ack_number == syn_num + 1 {
            break syn_ack_pack
        }
    };

    // 3. send ack
    let seq_num = syn_ack_pack.sequence_number;
    socket.send_to(&syn_ack_pack.create_ack().to_buffer(), addr)?;
    Ok(seq_num)
}

pub fn send_message(message: String, socket: &UdpSocket, addr: &str) -> Result<()> {
    socket.set_read_timeout(Some(Duration::new(1, 0)))?;
    let seq_num = establish_connection(socket, addr)?;
    let packets = string_to_packets(message, seq_num);

    // for every packet, monitor the status, I know this is overkill.
    let mut send_state = vec![];

    for packet in packets.iter() {
        send_state.push((packet.sequence_number, PacketStatus::Unsent))
    }
    let window_finish = min(TCP_WINDOW_LENGTH, packets.len() as u16);
    let mut window: Vec<u16> = (0..window_finish).into_iter().collect();
    let mut timeout_count = 0;

    loop {
        // if all packets ackd, close connection
        if (&send_state).into_iter().all(|(_, packet_status)| {
            match packet_status {
                PacketStatus::Acknowledged => true,
                _ => false
            }
        }) {
            return Ok(());
        }

        // send every packet in the window
        for i in &window {
            let item = send_state.get_mut(*i as usize).unwrap();
            let packet = packets.get(*i as usize).unwrap().clone();
            match &mut item.1 {
                // if the packet isn't sent, send it
                PacketStatus::Unsent => {
                    // println!("sent: {}", packet.sequence_number - seq_num);
                    let buf = packet.to_buffer();
                    socket.send_to(&buf, addr)?;

                    item.1 = PacketStatus::Sent { retry_time: get_retry_time(1), retry_count: 1 }
                },
                // if the packet is sent, check if time is up to send it again
                PacketStatus::Sent { retry_time, retry_count } => {
                    if *retry_count >= 5 {
                        return Err(Error::new(ErrorKind::ConnectionRefused, "Retry Count Exceeded"));
                    }
                    if *retry_time < Instant::now() {
                        *retry_count += 1;
                        println!("Resending packet: {} {}", packet.sequence_number, retry_count);
                        socket.send_to(&packet.to_buffer(), addr)?;

                        // increases 1 sec each time (probably too long but hey, should be fine for now)
                        // then add 0-1 sec of randomness so you can't get waves of usage that collectivly ddos a target
                        *retry_time = get_retry_time(*retry_count);
                    }
                },
                PacketStatus::Acknowledged => (),
            }
        }

        let mut buf = [0; MAX_PACKET_LENGTH];
        let socket_result = socket.recv_from(&mut buf);
        match socket_result {
            Ok(_) => timeout_count = 0,
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    timeout_count += 1;
                    if timeout_count > 3 {
                        return Err(e);
                    }
                    continue;
                }
                return Err(e);
            }
        }
        match TcpPacket::from_buffer(buf) {
            Ok(packet) => {
                if packet.flag_ack {
                    // now the inefficiency of this solution is really showing itself
                    let packet_index = send_state.binary_search_by(|(i, _)| i.cmp(&(packet.ack_number - 1))).unwrap();
                    let status = send_state.get_mut(packet_index).unwrap();
                    // set the incoming packet to be ackd
                    status.1 = PacketStatus::Acknowledged;
                    let mut window_start = *window.get(0).unwrap();

                    // find the next unackd packet, and set the window to be the appropriate range
                    for i in &window {
                        match send_state.get(*i as usize).unwrap().1 {
                            PacketStatus::Acknowledged => {
                                window_start = *i + 1;
                            }
                            _ => break,
                        }
                    }
                    let window_finish = min(window_start + TCP_WINDOW_LENGTH, packets.len() as u16);
                    window = (window_start..window_finish).into_iter().collect();
                }
            },
            // the only way the from buffer fails is if the checksum fails, in which case we need another message
            Err(_) => continue,
        }
        
    }
}
