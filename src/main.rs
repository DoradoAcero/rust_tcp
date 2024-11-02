use std::{net::UdpSocket, thread, time::Duration};

fn main() -> std::io::Result<()>{
    {
        let server_addr ="127.0.0.1:8000".to_string();
        let client_addr = "127.0.0.1:8001".to_string();
        setup_server(server_addr.clone());
        // wait for server
        thread::sleep(Duration::new(0, 500000000));

        let socket = UdpSocket::bind(client_addr).expect("couldn't bind to client address");
        socket.send_to(&[0,1,2,3,4,5,6,7,8,9], server_addr).expect("couldn't send data");
        let mut buf = [0; 10];
        socket.recv_from(&mut buf)?;
        println!("{:?}", buf);
    }
    Ok(())
}

fn setup_server(server_addr: String){
    thread::spawn(move || -> std::io::Result<()> {
            {
                let socket = UdpSocket::bind(server_addr)?;

                // Receives a single datagram message on the socket. If `buf` is too small to hold
                // the message, it will be cut off.
                let mut buf = [0; 10];
                let (amt, src) = socket.recv_from(&mut buf)?;

                // Redeclare `buf` as slice of the received data and send reverse data back to origin.
                println!("{:?}", buf);
                let buf = &mut buf[..amt];
                buf.reverse();
                socket.send_to(buf, &src)?;
            } // the socket is closed here
            Ok(())
        }
    );
}