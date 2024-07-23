/// A program that tests the speed at which your computer can send arbitrary UDP packets.

use top_down_shooter::common::*;
use std::net::UdpSocket;
use std::time::*;

fn main() {
  let server_ip = "192.168.1.8";
  let server_ip: String = format!("{}:{}", server_ip, SERVER_LISTEN_PORT);
  let sending_ip: String = format!("0.0.0.0:{}", CLIENT_SEND_PORT);
  let sending_socket = UdpSocket::bind(sending_ip).expect("Could not bind client sender socket");
  loop {

    let mut counter: Instant = Instant::now();

    let client_packet: ClientPacket = ClientPacket {
      position:    Vector2 {x: 0.0, y: 0.0 },
      aim_direction: Vector2 { x: 0.0, y: 0.0 },
      shooting: false,
      shooting_secondary: false,
      };
      
    let serialized: Vec<u8> = bincode::serialize(&client_packet).expect("Failed to serialize message");
    sending_socket.send_to(&serialized, server_ip.clone()).expect("Failed to send packet to server.");
    
    println!("Packet Frequency: {} Hz", 1.0/counter.elapsed().as_secs_f64());
    counter = Instant::now()
  }
}