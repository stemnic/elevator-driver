pub mod net_io;

use std::io;

use net_io::Communication;
use net_io::RequestType;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

pub struct ElevIo {
    pub io: Communication,
    to_elevator: Sender<(RequestType , std::vec::Vec<u8>)>,
    to_elevator_feedback: Sender<Sender<std::vec::Vec<u8>>>,
}

#[derive(Copy, Clone, Debug)]
pub enum Floor {
    At(u8),
    Between,
}
pub const N_FLOORS: u8 = 4;
const TOP: u8 = N_FLOORS - 1;
const SEC_TOP: u8 = N_FLOORS - 2;

#[derive(Copy, Clone, Debug)]
pub enum Button {
    CallUp(Floor),
    CallDown(Floor),
    Internal(Floor),
}

#[derive(Copy, Clone, Debug)]
pub enum MotorDir {
    Up,
    Down,
    Stop,
}

#[derive(Copy, Clone, Debug)]
pub enum Light {
    On,
    Off,
}

#[derive(Copy, Clone, Debug)]
pub enum Signal {
    High,
    Low,
}

struct sender_type {
    sender: Sender<std::vec::Vec<u8>>,
    data: std::vec::Vec<u8>
}

impl Signal {
    pub fn new(value: usize) -> Self {
        if value == 0 { Signal::Low }
        else          { Signal::High }
    }
}

pub const DEFAULT_IP_ADDRESS: &str = "localhost";
pub const DEFAULT_PORT: u16 = 15657;

impl ElevIo {
    pub fn new(ip_address: &str, port: u16) -> io::Result<Self> {
        let (to_elev_sender, to_elev_reciver) = channel::<(RequestType , std::vec::Vec<u8>)>();
        let (from_elev_sender, from_elev_reciver) = channel::<std::vec::Vec<u8>>();
        let (send_data, receive_data) = channel::<Sender<std::vec::Vec<u8>>>();
        let elev = ElevIo { io: Communication::new(String::from(ip_address), port, to_elev_reciver, from_elev_sender)?, to_elevator: to_elev_sender, to_elevator_feedback: send_data};
        elev.set_all_light(Light::Off)?;
        elev.set_floor_light(Floor::At(0))?;
        // Thread spawning receive incomming polling data
        thread::spawn(move || {
            loop {
                match receive_data.recv() {

                    Ok(channel_sender) => {
                        let data = match from_elev_reciver.recv() {
                            Ok(value) => {
                                //println!("[elev_driver] Answer:  {:?}", value);
                                value
                            },
                            Err(error) => {
                                println!("[elev_driver] Recv Error {:?}", error);
                                vec![0,0,0,0]
                            }
                        };
                        channel_sender.send(data).unwrap();
                    }
                    Err(error) => {
                        panic!("[elev_driver] Error receiveing data! {:?}", error);
                    }

                }

            }
        });
        Ok(elev)
    }

    pub fn set_motor_dir(&self, dir: MotorDir) -> io::Result<()> {
        match dir {
            MotorDir::Stop => {
                let dir = vec![1, 0, 0, 0];
                self.to_elevator.send((RequestType::Write , dir));
            },
            MotorDir::Up => {
                let dir = vec![1, 1, 0, 0];
                self.to_elevator.send((RequestType::Write , dir));
            },
            MotorDir::Down => {
                let dir = vec![1, 255, 0, 0];
                self.to_elevator.send((RequestType::Write , dir));
            },
        };
        Ok(())
    }

    pub fn set_all_light(&self, mode: Light) -> io::Result<()> {
        for floor in 0..N_FLOORS {
            if floor != TOP { self.set_button_light(Button::CallUp(Floor::At(floor)), mode)?; }
            if floor != 0   { self.set_button_light(Button::CallDown(Floor::At(floor)), mode)?; }
            self.set_button_light(Button::Internal(Floor::At(floor)), mode)?;
        }
        self.set_stop_light(mode)?;
        self.set_door_light(mode)?;
        Ok(())
    }

    pub fn set_button_light(&self, button: Button, mode: Light) -> io::Result<()> {
        let mut light_command: std::vec::Vec<u8> = vec![2, 0, 0, 0];
        let addr = match button {
            Button::CallUp(Floor::At(floor @ 0..=SEC_TOP)) => {
                light_command[1] = 0;
                light_command[2] = floor;
            },
            Button::CallDown(Floor::At(floor @ 1..=TOP)) => {
                light_command[1] = 1;
                light_command[2] = floor;
            },
            Button::Internal(Floor::At(floor @ 0..=TOP)) => {
                light_command[1] = 2;
                light_command[2] = floor;
            },
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "given floor is not supported for given button")),
        };
        match mode {
            Light::On => {
                light_command[3] = 1;
            },
            Light::Off => {
                light_command[3] = 0;
            },
        };
        self.to_elevator.send((RequestType::Write , light_command));
        Ok(())
        
    }

    pub fn set_floor_light(&self, floor: Floor) -> io::Result<()> {
        if let Floor::At(etg) = floor {
            if etg > TOP {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "given floor is not supported"));
            }
            let floor_vec = vec![3, etg, 0, 0];
            self.to_elevator.send((RequestType::Write , floor_vec));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot set light between floors"))
        }
    }

    pub fn set_door_light(&self, mode: Light) -> io::Result<()> {
        match mode {
            Light::On => {
                self.to_elevator.send((RequestType::Write , vec![4, 1, 0, 0]));
            },
            Light::Off => {
                self.to_elevator.send((RequestType::Write , vec![4, 0, 0, 0]));
            },
        }
        Ok(())
    }

    pub fn set_stop_light(&self, mode: Light) -> io::Result<()> {
        match mode {
            Light::On => {
                self.to_elevator.send((RequestType::Write, vec![5, 1, 0, 0]));
            },
            Light::Off => {
                self.to_elevator.send((RequestType::Write, vec![5, 0, 0, 0]));
            },
        }
        Ok(())
    }

    pub fn get_button_signal(&self, button: Button) -> io::Result<Signal> {
        let mut light_command: std::vec::Vec<u8> = vec![6, 0, 0, 0];
        match button {
            Button::CallUp(Floor::At(floor @ 0..=SEC_TOP)) => {
                light_command[1] = 0;
                light_command[2] = floor;
            },
            Button::CallDown(Floor::At(floor @ 1..=TOP)) => {
                light_command[1] = 1;
                light_command[2] = floor;
            },
            Button::Internal(Floor::At(floor @ 0..=TOP)) => {
                light_command[1] = 2;
                light_command[2] = floor;
            },
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "given floor is not supported for given button")),
        };
        let (sender, receive) = channel::<std::vec::Vec<u8>>();
        self.to_elevator_feedback.send(sender).unwrap();
        self.to_elevator.send((RequestType::Read , light_command)).unwrap();
        match receive.recv(){
            Ok(value) => {
                if value[1] == 0 {
                    return Ok(Signal::Low)
                }
                return Ok(Signal::High)
            }
            Err(_) => {
                Ok(Signal::Low)
            }
        }
    }
    
    pub fn get_floor_signal(&self) -> io::Result<Floor> {
        
        let (sender, receive) = channel::<std::vec::Vec<u8>>();
        let get_floor_vec = vec![7, 0, 0, 0];
        self.to_elevator_feedback.send(sender).unwrap();
        self.to_elevator.send((RequestType::Read, get_floor_vec)).unwrap();
        match receive.recv(){
            Ok(value) => {
                if value[1] == 0 {
                    return Ok(Floor::Between)
                }
                return Ok(Floor::At(value[2]))
            }
            Err(_) => {
                Ok(Floor::Between)
            }
        }
        
    }
    
    pub fn get_stop_signal(&self) -> io::Result<Signal> {
        let (sender, receive) = channel::<std::vec::Vec<u8>>();
        let get_floor_vec = vec![8, 0, 0, 0];
        self.to_elevator_feedback.send(sender).unwrap();
        self.to_elevator.send((RequestType::Read, get_floor_vec)).unwrap();
        match receive.recv(){
            Ok(value) => {
                if value[1] == 0 {
                    return Ok(Signal::Low)
                }
                return Ok(Signal::High)
            }
            Err(_) => {
                Ok(Signal::Low)
            }
        }
    }

    pub fn get_obstr_signal(&self) -> io::Result<Signal> {
        let (sender, receive) = channel::<std::vec::Vec<u8>>();
        let get_floor_vec = vec![9, 0, 0, 0];
        self.to_elevator_feedback.send(sender).unwrap();
        self.to_elevator.send((RequestType::Read, get_floor_vec)).unwrap();
        match receive.recv(){
            Ok(value) => {
                if value[1] == 0 {
                    return Ok(Signal::Low)
                }
                return Ok(Signal::High)
            }
            Err(_) => {
                Ok(Signal::Low)
            }
        }
    }
    
}
