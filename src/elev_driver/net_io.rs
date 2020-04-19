use std::io;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::{thread, time};

#[derive(Debug)]
pub enum RequestType {
    Write,
    Read
}

pub struct Communication {
    data_to_recive: Sender<std::vec::Vec<u8>>,
    pub lifeline: std::thread::JoinHandle<()>,
}

impl Communication {
    pub fn new(ip: String, port: u16, send_message: Receiver<(RequestType , std::vec::Vec<u8>)>, receive_message: Sender<std::vec::Vec<u8>>) -> io::Result<Self> {
        let connect_ip: String = ip + ":" + &port.to_string();
        let copy_reciver = receive_message.clone();
        let network_lifeline = thread::spawn(move || { // Keeps the network socket thread alive
            let mut stream = TcpStream::connect(connect_ip).expect("Could not connect to Elevator server");
            loop {
                match send_message.recv() {
                    Ok(data) => {
                        let (req_type, msg) = data;
                        match req_type {
                            RequestType::Write => {
                                let _ = stream.write(&msg.into_boxed_slice());
                            }
                            RequestType::Read => {
                                let mut buffer = [0; 4];
                                let _ = stream.write(&msg.into_boxed_slice()); //We always expect a response by sending a read command to the elevator server
                                let _ = match stream.read(&mut buffer) {
                                    Ok(data) => {
                                        data
                                    }
                                    Err(_) => {
                                        0
                                    }
                                };
                                let mut t = Vec::new();
                                for i in buffer.iter() { t.push(*i); }
                                let _ = copy_reciver.send(t);
                            }
                        }
                    }
        
                    Err(_) => {
                        println!("[elev_driver] TCP connection closed");
                        return
                    }
                }
            }
        });
        Ok(Communication{data_to_recive: receive_message, lifeline: network_lifeline})
    }
}