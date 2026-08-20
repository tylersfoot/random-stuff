use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;


fn main() {
    println!("Starting server...");
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Could not bind socket");
    let target_address = "127.0.0.1:5000";
    let mut mock_x = 0.0_f32;

    loop {
        let msg = format!("{:.3},{},{}", mock_x.sin() / 8.0, 0.9, 7.2);

        socket.send_to(msg.as_bytes(), target_address).expect("Failed to send packet");
        // println!("Sent: {msg}");

        mock_x += 0.1;
        sleep(Duration::from_millis(16));
    }
}

