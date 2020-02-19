use std::io;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::thread;

pub struct Communication {
    data_to_recive: Sender<std::vec::Vec<u8>>,
    pub lifeline: std::thread::JoinHandle<()>,
}

impl Communication {
    pub fn new(ip: String, port: u16, send_message: Receiver<std::vec::Vec<u8>>, receive_message: Sender<std::vec::Vec<u8>>) -> io::Result<Self> {
        let connect_ip: String = ip + ":" + &port.to_string();
        let copy_reciver = receive_message.clone();
        let network_lifeline = thread::spawn(move || {
            let mut stream = TcpStream::connect(connect_ip).unwrap();
            stream.set_read_timeout(Some(Duration::from_millis(10))).unwrap(); //Timeout since the elevator server does not have a ACK
            loop {
                match send_message.recv() {
                    Ok(data) => {
                        println!("Sending to elevator: {:?}", data);
                        let mut buffer = [0; 4];
                        let _ = stream.write(&data.into_boxed_slice()); //We always expect a response by polling the elevator server
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
        
                    Err(_) => return, // This means, that the sender has disconnected
                                      // and no further messages can ever be received
                }
            }
        });
        Ok(Communication{data_to_recive: receive_message, lifeline: network_lifeline})
    }
}