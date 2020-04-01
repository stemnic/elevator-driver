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
        let network_lifeline = thread::spawn(move || {
            let mut stream = TcpStream::connect(connect_ip).expect("Could not connect to Elevator server");
            //stream.set_read_timeout(Some(Duration::from_millis(10))).expect("Failed to set read timeout for tcp stream"); //Timeout since the elevator server does not have a ACK
            loop {
                match send_message.recv() {
                    Ok(data) => {
                        let (req_type, msg) = data;
                        //println!("[elev_driver] Type: {:?} Sending: {:?}", req_type, msg);
                        match req_type {
                            RequestType::Write => {
                                let _ = stream.write(&msg.into_boxed_slice());
                            }
                            RequestType::Read => {
                                //thread::sleep(time::Duration::from_millis(20));
                                let mut buffer = [0; 4];
                                let _ = stream.write(&msg.into_boxed_slice()); //We always expect a response by polling the elevator server
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
                                //println!("[elev_driver] Recived: {:?}", t);
                                let _ = copy_reciver.send(t);
                            }
                        }
                    }
        
                    Err(_) => {
                        println!("[elev_driver] TCP connection closed");
                        return
                    } // This means, that the sender has disconnected
                      // and no further messages can ever be received
                }
            }
        });
        Ok(Communication{data_to_recive: receive_message, lifeline: network_lifeline})
    }
}