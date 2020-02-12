use std::io;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

pub struct Communication {
    stream: TcpStream,
    pub tx: Sender<u8>,
}

impl Communication {
    pub fn new(ip: String, port: u8, tx: Sender<u8>) -> io::Result<Self> {
        let connect_ip: String = ip + ":" + &port.to_string();
        let mut tcp_stream = TcpStream::connect(connect_ip)?;

        Ok(Communication{ stream: tcp_stream, tx: tx})
    }
    pub fn send(&mut self, buffer: &[u8]) -> io::Result<()> {
        self.stream.write(buffer)?;
        Ok(())
    }

    pub fn listen(&mut self) {
        let mut readBuffer = [0 ; 9];
        loop {
            match self.stream.read(&mut readBuffer) {
                Ok(size) => {
                    
                }
                Err(_) => return,
            }
            
        }
    }
}