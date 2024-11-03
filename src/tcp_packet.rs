pub const MAX_DATA_LENGTH: usize = 1380; // https://superuser.com/questions/1341012/practical-vs-theoretical-max-limit-of-tcp-packet-size
pub const MAX_PACKET_LENGTH: usize = MAX_DATA_LENGTH + 16; // this is assuming no options
pub const TCP_WINDOW_LENGTH: u16 = 10; // havent decided on what this should be yet

// https://datatracker.ietf.org/doc/html/rfc9293#name-header-format
#[derive(Debug, Clone)]
pub struct TcpPacket {
    // source_port: u16, udp is handling this one for me
    // destination_port: u16, and this one
    pub sequence_number: u32,
    pub ack_number: u32,
    pub data_offset: u8, //actually a u4, but this doesn't exist in rust
    pub flag_congestion_window_reduced: bool, //not gonna do this one
    pub flag_echo_explicit_congestion_notification: bool, //or this
    pub flag_urgent_pointer: bool, //TODO this is a maybe, could be fun
    pub flag_ack: bool,
    pub flag_push: bool,
    pub flag_reset: bool,
    pub flag_sync_seq_numbers: bool,
    pub flag_finished: bool,
    pub window: u16,
    pub urgent_pointer: u16,
    pub options: Vec<u8>,
    pub data: Vec<u8>,
}

use std::io::{Error, ErrorKind};

use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;

impl TcpPacket {
    pub fn from_buffer(buf: [u8; MAX_PACKET_LENGTH]) -> Result<TcpPacket, Error> {
        let mut options = vec![];
        let mut data = vec![];
        let data_offset = buf[8];
        let offset_multiplier = 32 / 8;
        let mut i: usize = 4 * offset_multiplier; // where to start the options loop
        while i < data_offset as usize * offset_multiplier {
            options.push(buf[i]);
            i += 1;
        }

        while i < buf.len() {
            data.push(buf[i]);
            i += 1;
        }

        // checksum calculation
        let mut sum: u16 = 0;
        for i in 0..buf.len() {
            sum = sum.wrapping_add(if i % 2 == 0 {
                buf[i] as u16
            } else {
                (buf[i] as u16) << 8
            });
        }

        if sum != 0xffff {
            return Err(Error::new(ErrorKind::InvalidData, "Checksum failed"));
        }

        Ok(TcpPacket {
            sequence_number: LittleEndian::read_u32(&buf),
            ack_number: LittleEndian::read_u32(&buf[4..]),
            data_offset,
            flag_congestion_window_reduced: buf[9] & (1 << 7) != 0,
            flag_echo_explicit_congestion_notification: buf[9] & (1 << 6) != 0,
            flag_urgent_pointer: buf[9] & (1 << 5) != 0,
            flag_ack: buf[9] & (1 << 4) != 0,
            flag_push: buf[9] & (1 << 3) != 0,
            flag_reset: buf[9] & (1 << 2) != 0,
            flag_sync_seq_numbers: buf[9] & (1 << 1) != 0,
            flag_finished: buf[9] & 1 != 0,
            window: LittleEndian::read_u16(&buf[10..]),
            urgent_pointer: LittleEndian::read_u16(&buf[14..]), // leave this as some free space for later
            options,
            data,
        })
    }

    pub fn to_buffer<'a>(self) -> [u8; MAX_PACKET_LENGTH] {
        let mut buf = [0; MAX_PACKET_LENGTH]; // this len is assuming no options
        LittleEndian::write_u32(&mut buf, self.sequence_number); // 0..4
        LittleEndian::write_u32(&mut buf[4..], self.ack_number); //4..8
        buf[8] = self.data_offset; // + reserved
        let control_bits: u8 = {
            ((self.flag_congestion_window_reduced as u8) << 7)
                + ((self.flag_echo_explicit_congestion_notification as u8) << 6)
                + ((self.flag_urgent_pointer as u8) << 5)
                + ((self.flag_ack as u8) << 4)
                + ((self.flag_push as u8) << 3)
                + ((self.flag_reset as u8) << 2)
                + ((self.flag_sync_seq_numbers as u8) << 1)
                + (self.flag_finished as u8)
        };
        buf[9] = control_bits;
        LittleEndian::write_u16(&mut buf[10..], self.window); //10..12
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

        let mut sum: u16 = 0;
        for i in 0..buf.len() {
            sum = sum.wrapping_add(if i % 2 == 0 {
                buf[i] as u16
            } else {
                (buf[i] as u16) << 8
            });
        }
        let checksum = 0xffff - sum;
        LittleEndian::write_u16(&mut buf[12..], checksum); // 12..14

        buf
    }

    pub fn create_ack(self) -> TcpPacket {
        let mut ack_pack = self.clone();

        ack_pack.ack_number = ack_pack.sequence_number + 1;
        ack_pack.flag_sync_seq_numbers = false; // this is for the init ack from the syn-ack
        ack_pack.flag_ack = true;
        ack_pack.data.clear();

        ack_pack
    }

    pub fn create_syn_ack(self) -> TcpPacket {
        let mut rng = rand::thread_rng();
        let mut ack_pack = self.clone();
        ack_pack.flag_ack = true;
        ack_pack.ack_number = ack_pack.sequence_number + 1;
        ack_pack.sequence_number = rng.gen();

        ack_pack
    }
}

pub fn string_to_packets(message: String, seq_num: u32) -> Vec<TcpPacket> {
    let mut packets = vec![];
    for (i, packet_data) in message.as_bytes().chunks(MAX_DATA_LENGTH).enumerate() {
        let sequence_number = seq_num.wrapping_add(i as u32);
        packets.push(TcpPacket {
            sequence_number,
            ack_number: 0,
            data_offset: 4, // no options for now, starts at 5 as i am missing ports in my protocol
            flag_congestion_window_reduced: false,
            flag_echo_explicit_congestion_notification: false,
            flag_urgent_pointer: false,
            flag_ack: false,
            flag_push: false,
            flag_finished: false,
            flag_reset: false,
            flag_sync_seq_numbers: false,
            window: TCP_WINDOW_LENGTH,
            urgent_pointer: 0, // leave this as some free space for later
            options: vec![],   // imma leave options empty for now
            data: packet_data.to_vec(),
        });
    }

    let last_packet = packets.last_mut().unwrap(); // last packet
    last_packet.flag_finished = true;

    packets
}

pub fn create_syn_packet() -> TcpPacket {
    let mut rng = rand::thread_rng();
    TcpPacket {
        sequence_number: rng.gen(),
        ack_number: 0,
        data_offset: 4,
        flag_congestion_window_reduced: false,
        flag_echo_explicit_congestion_notification: false,
        flag_urgent_pointer: false,
        flag_ack: false,
        flag_push: false,
        flag_reset: false,
        flag_sync_seq_numbers: true,
        flag_finished: false,
        window: TCP_WINDOW_LENGTH,
        urgent_pointer: 0,
        options: vec![],
        data: vec![],
    }
}