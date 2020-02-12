use std::io;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::mpsc::{Receiver, Sender};

pub struct Communication {
    stream: TcpStream,
    data_to_send: Receiver<std::vec::Vec<u8>>,
    data_to_recive: Sender<std::vec::Vec<u8>>
}

impl Communication {
    pub fn new(ip: String, port: u16, send_message: Receiver<std::vec::Vec<u8>>, receive_message: Sender<std::vec::Vec<u8>>) -> io::Result<Self> {
        let connect_ip: String = ip + ":" + &port.to_string();
        let tcp_stream = TcpStream::connect(connect_ip)?;

        Ok(Communication{ stream: tcp_stream, data_to_send: send_message, data_to_recive: receive_message})
    }

    pub fn start(&mut self){
        loop {
            match self.data_to_send.recv() {
                Ok(data) => {
                    let mut buffer = [0; 1024];
                    let _ = self.stream.write(&data.into_boxed_slice());
                    let _ = self.stream.read(&mut buffer);
                    let mut t = Vec::new();
                    for i in buffer.iter() { t.push(*i); }
                    let _ = self.data_to_recive.send(t);
                }
    
                Err(_) => return, // This means, that the sender has disconnected
                                  // and no further messages can ever be received
            }
        }
    }
}