
// https://datatracker.ietf.org/doc/html/rfc9293#name-header-format
pub struct TcpPacket {
    // source_port: u16, udp is handling this one for me
    // destination_port: u16, and this one
    sequence_number: u32,
    ack_number: u32,
    data_offset: u8, //actually a u4, but this doesn't exist in rust
    // flag_congestion_window_reduced: bool, not gonna do this one
    // flag_echo_explicit_congestion_notification: bool, or this
    // flag_urgent_pointer: bool, TODO this is a maybe, could be fun
    flag_ack: bool,
    flag_push: bool,
    flag_reset: bool,
    flag_sync_seq_numbers: bool,
    flag_finished: bool,
    window: u16,
    checksum: u16,
    urgent_pointer: u16,
    options: Vec<u32>,
    data: Vec<u32>,
}



