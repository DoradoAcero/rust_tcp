use std::{io::{Error, ErrorKind, Result}, net::UdpSocket, time::{Duration, Instant}};
use rand::Rng;

use crate::tcp_packet::{string_to_packets, TcpPacket, MAX_PACKET_LENGTH, TCP_WINDOW_LENGTH};

enum PacketStatus {
    Unsent,
    Acknowledged,
    Sent{retry_time: Instant, retry_count: u32},
}

pub fn send_message(message: String, socket: UdpSocket, addr: &str) -> Result<()> {
    let packets = string_to_packets(message);
    let mut send_state = vec![];

    for (i, _) in packets.iter().enumerate() {
        send_state.push((i, PacketStatus::Unsent))
    }
    let mut window: Vec<u16> = (0..TCP_WINDOW_LENGTH).into_iter().collect();

    loop {
        if (&send_state).into_iter().all(|(_, packet_status)| {
            match packet_status {
                PacketStatus::Acknowledged => true,
                _ => false
            }
        }) {
            return Ok(());
        }

        for i in &window {
            let item = send_state.get(*i as usize).unwrap();
            let packet = packets.get(*i as usize).unwrap().clone();
            match item.1 {
                PacketStatus::Unsent => {
                    let buf = packet.to_buffer();
                    socket.send_to(&buf, addr)?;
                },
                PacketStatus::Sent { mut retry_time, mut retry_count } => {
                    if retry_count > 5 {
                        return Err(Error::new(ErrorKind::ConnectionRefused, "Retry Count Exceeded"));
                    }
                    if retry_time < Instant::now() {
                        retry_count += 1;
                        socket.send_to(&packet.to_buffer(), addr)?;

                        // increases 1 sec each time (probably too long but hey, should be fine for now)
                        // then add 0-1 sec of randomness for non overlapping stuffs
                        retry_time = Instant::now() + Duration::new(retry_count as u64, rand::thread_rng().gen_range(0..1000000000))
                    }
                },
                PacketStatus::Acknowledged => (),
            }
        }

        let mut buf = [0; MAX_PACKET_LENGTH];
        socket.recv_from(&mut buf)?;
        let recieved_packet = TcpPacket::from_buffer(buf);
        if recieved_packet.
    }
}