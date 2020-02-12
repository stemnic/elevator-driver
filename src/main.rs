use std::sync::mpsc::channel;
use std::string;
use std::thread;

mod net_io;
pub mod elev_io;

use net_io::*;

fn main() {
    let (sending_tx, sending_rx) = channel::<std::vec::Vec<u8>>();
    let (reciving_tx, reciving_rx) = channel::<std::vec::Vec<u8>>();
    let net = thread::spawn(move || {
        let mut network = Communication::new("localhost".to_string(), 12345, sending_rx, reciving_tx).unwrap();
        network.start();
    });
    sending_tx.send(vec![0,0,0,1]).unwrap();
    loop {
        println!("Got something! {:?}", reciving_rx.recv().unwrap())
    }
    net.join();
}
