use std::sync::mpsc::channel;
use std::string;
use std::thread;
use std::time;
mod elev_driver;

use elev_driver::net_io::*;
use elev_driver::*;

fn main() {
    let driver = elev_driver::ElevIo::new(DEFAULT_IP_ADDRESS, DEFAULT_PORT).unwrap();

    const SEC_TOP: u8 = elev_driver::N_FLOORS - 1;
    loop {
        thread::sleep_ms(10);
        for floor in 0..elev_driver::N_FLOORS {
            match driver.get_button_signal(elev_driver::Button::Internal(elev_driver::Floor::At(floor))).expect("Button signal error") {
                Signal::High => {
                    println!("[elev_driver] Going to {:?}", floor);
                    loop {
                        match driver.get_floor_signal()
                                    .expect("Get FloorSignal failed") {
                            Floor::At(data) => {
                                if data > floor{
                                    driver.set_motor_dir(MotorDir::Down).expect("Set MotorDir failed");
                                }
                                if data < floor{
                                    driver.set_motor_dir(MotorDir::Up).expect("Set MotorDir failed");
                                }
                                if data == floor{
                                    driver.set_motor_dir(MotorDir::Stop).expect("Set MotorDir failed");
                                    break;
                                }
                            }
                            Floor::Between => {

                            }
                        }
                        
                    }
                }
                Signal::Low => {

                }
            }
        }
        
        if let Signal::High = driver.get_stop_signal().expect("Get StopSignal failed") {
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

#[cfg(test)]
mod tests {
    #[test]
    fn exploration() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn another() {
        panic!("Make this test fail");
    }
}