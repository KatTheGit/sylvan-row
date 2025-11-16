use std::{io::Write, net::{TcpListener, TcpStream}};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:25569")
      .expect("hor hor hor fnaf");

    stream.write(&[1])
      .expect("oopsies");

}