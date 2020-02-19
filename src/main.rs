use std::sync::mpsc::channel;
use std::string;
use std::thread;

mod elev_driver;

use elev_driver::net_io::*;
use elev_driver::*;

fn main() {
    let driver = elev_driver::ElevIo::new().unwrap();
    driver.set_motor_dir(elev_driver::MotorDir::Up);
    driver.set_stop_light(elev_driver::Light::On);

    driver.io.lifeline.join();
    /*
    let (sending_tx, sending_rx) = channel::<std::vec::Vec<u8>>();
    let (reciving_tx, reciving_rx) = channel::<std::vec::Vec<u8>>();
    let net = thread::spawn(move || {
        let mut network = Communication::new("localhost".to_string(), 15657, sending_rx, reciving_tx).unwrap();
        network.start();
    });
    sending_tx.send(vec![6,0,0,1]).unwrap();
    loop {
        println!("Got something! {:?}", reciving_rx.recv().unwrap())
    }
    net.join();
    */
}
