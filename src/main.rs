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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::*;
    use std::io::*;

    fn tcp_response( expectedData: std::vec::Vec<u8> ) {
        thread::spawn(move || {
            let listener = TcpListener::bind("localhost:15657").unwrap();
            for stream in listener.incoming() {
                let mut data = [0; 4];
                let mut stream = stream.unwrap();
                while match stream.read(&mut data) {
                    Ok(size) => {
                        stream.write(&expectedData.clone().into_boxed_slice()).unwrap();
                        true
                    },
                    Err(_) => {
                        println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                        stream.shutdown(Shutdown::Both).unwrap();
                        false
                    }
                } {}
            }
        });
    }
    #[test]
    fn test_floor_signal() {
        for floor in 0..N_FLOORS {
            tcp_response(vec![7,1,floor,0]);
            let driver = elev_driver::ElevIo::new(DEFAULT_IP_ADDRESS, DEFAULT_PORT).unwrap();
            assert_eq!(elev_driver::Floor::At(0), driver.get_floor_signal().unwrap())
        }
    }
}