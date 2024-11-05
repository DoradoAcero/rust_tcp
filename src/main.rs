use std::{fs, io::Result, net::UdpSocket, thread::{self, sleep}, time::{Duration, Instant}};

use client::send_message;
mod tcp_packet;
mod client;
mod server;
mod utils;
use port::TcpPort;
use server::setup_server;
mod port;

fn main() -> Result<()>{
    {
        let send_counts = 3;
        let server_addr ="127.0.0.1:8000".to_string();
        let client_addr = "127.0.0.1:8001".to_string();
        setup_server(server_addr.clone());
        // wait for server, 0.5s
        thread::sleep(Duration::new(0, 500000000));

        let contents = fs::read_to_string("bee_movie.txt")
            .expect("Should have been able to read the file");

        // let contents = (0..3000).map(|_| "!").collect::<String>(); // just something to make 2 packets

        let socket = TcpPort::new(&client_addr).expect("couldn't bind to client address");
        let start = Instant::now();
        for _ in 0..send_counts {
            socket.send(contents.clone(), &server_addr)?;
        }
        let time_taken = start.elapsed();
        println!("time taken: {:.2?}: {:.2?}", time_taken, time_taken/send_counts);
        sleep(Duration::new(1,0)); // let the server finish recieving
    }
    Ok(())
}
