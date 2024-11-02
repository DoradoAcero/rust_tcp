const MAX_PACKET_LENGTH: usize = 1000;// https://superuser.com/questions/1341012/practical-vs-theoretical-max-limit-of-tcp-packet-size
const TCP_WINDOW_LENGTH: u16 = 10; // havent decided on what this should be yet

// https://datatracker.ietf.org/doc/html/rfc9293#name-header-format
pub struct TcpPacket {
    // source_port: u16, udp is handling this one for me
    // destination_port: u16, and this one
    sequence_number: u32,
    ack_number: u32,
    data_offset: u8, //actually a u4, but this doesn't exist in rust
    flag_congestion_window_reduced: bool, //not gonna do this one
    flag_echo_explicit_congestion_notification: bool, //or this
    flag_urgent_pointer: bool, //TODO this is a maybe, could be fun
    flag_ack: bool,
    flag_push: bool,
    flag_reset: bool,
    flag_sync_seq_numbers: bool,
    flag_finished: bool,
    window: u16,
    checksum: u16,
    urgent_pointer: u16,
    options: Vec<u8>,
    data: Vec<u8>,
}

use byteorder::{ByteOrder, LittleEndian};


impl TcpPacket {
    pub fn to_buffer(self) -> [u8; MAX_PACKET_LENGTH] {
        let mut buf = [0; MAX_PACKET_LENGTH];
        LittleEndian::write_u32(&mut buf, self.sequence_number); // 0..4
        LittleEndian::write_u32(&mut buf[4..], self.ack_number); //4..8
        buf[8] = self.data_offset; // + reserved
        let control_bits: u8 = {
            (self.flag_congestion_window_reduced as u8) << 7 +
            (self.flag_echo_explicit_congestion_notification as u8) << 6 + 
            (self.flag_urgent_pointer as u8) << 5 + 
            (self.flag_ack as u8) << 4 + 
            (self.flag_push as u8) << 3 + 
            (self.flag_reset as u8) << 2 + 
            (self.flag_sync_seq_numbers as u8) << 1 + 
            (self.flag_finished as u8) 
        };
        buf[9] = control_bits;
        LittleEndian::write_u16(&mut buf[10..], self.window); //10..12
        LittleEndian::write_u16(&mut buf[12..], self.checksum); // 12..14
        LittleEndian::write_u16(&mut buf[14..], self.urgent_pointer); // 14..16
        let mut i = 16;
        for option in self.options {
            buf[i] = option;
            i += 1;
        }

        for utf8 in self.data {
            buf[i] = utf8;
            i += 1;
        }

        buf
    }
}


pub fn string_to_packets(message: String) -> Vec<TcpPacket> {
    let mut packets = vec![];
    for (i, packet_data) in message.as_bytes().chunks(MAX_PACKET_LENGTH).enumerate() {
        packets.push(TcpPacket {
            sequence_number: i as u32,
            ack_number: 0,
            data_offset: 0, // TODO idk what to do with this quite yet
            flag_congestion_window_reduced: false,
            flag_echo_explicit_congestion_notification: false,
            flag_urgent_pointer: false,
            flag_ack: false,
            flag_push: false,
            flag_finished: false,
            flag_reset: false,
            flag_sync_seq_numbers: false,
            window: TCP_WINDOW_LENGTH,
            checksum: 0, // TODO
            urgent_pointer: 0, // leave this as some free space for later
            options: vec![], // imma leave options empty for now
            data: packet_data.to_vec(),
        });
    }
    
    packets
}
