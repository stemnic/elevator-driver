use std::sync::mpsc::channel;
use std::string;
use std::thread;
use std::time;
mod elev_driver;

use elev_driver::net_io::*;
use elev_driver::*;

fn main() {
    let driver = elev_driver::ElevIo::new().unwrap();
    //driver.set_motor_dir(elev_driver::MotorDir::Up);
    //driver.set_stop_light(elev_driver::Light::On);
    //let floor = driver.get_floor_signal().unwrap();
    //println!("At floor: {:?}", floor);
    //println!("Button sig: {:?}", driver.get_button_signal(elev_driver::Button::CallDown(Floor::At(3))).unwrap());
    //println!("stop sig: {:?}", driver.get_stop_signal().unwrap());

    driver.set_motor_dir(MotorDir::Up)
           .expect("Set MotorDir failed");

    const SEC_TOP: u8 = elev_driver::N_FLOORS - 1;
    loop {
        thread::sleep(time::Duration::from_millis(50));
        
        match driver.get_floor_signal()
                     .expect("Get FloorSignal failed") {
            Floor::At(SEC_TOP) => driver.set_motor_dir(MotorDir::Down)
                                         .expect("Set MotorDir failed"),
            Floor::At(0) => driver.set_motor_dir(MotorDir::Up)
                                   .expect("Set MotorDir Failed"),
            _ => {}
        }
        thread::sleep(time::Duration::from_millis(50));

        if let Signal::High = driver.get_stop_signal()
                                    .expect("Get StopSignal failed") {
                                    driver.set_motor_dir(MotorDir::Stop)
                   .expect("Set MotorDir failed");
            return;
        }
    }

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
