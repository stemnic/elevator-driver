pub mod net_io;

use std::io;

use net_io::Communication;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

pub struct ElevIo {
    pub io: Communication,
    to_elevator: Sender<std::vec::Vec<u8>>,
}

#[derive(Copy, Clone)]
pub enum Floor {
    At(u8),
    Between,
}
pub const N_FLOORS: u8 = 4;
const TOP: u8 = N_FLOORS - 1;
const SEC_TOP: u8 = N_FLOORS - 2;

#[derive(Copy, Clone)]
pub enum Button {
    CallUp(Floor),
    CallDown(Floor),
    Internal(Floor),
}

#[derive(Copy, Clone)]
pub enum MotorDir {
    Up,
    Down,
    Stop,
}

#[derive(Copy, Clone)]
pub enum Light {
    On,
    Off,
}

#[derive(Copy, Clone)]
pub enum Signal {
    High,
    Low,
}

impl Signal {
    pub fn new(value: usize) -> Self {
        if value == 0 { Signal::Low }
        else          { Signal::High }
    }
}

const IP_ADDRESS: &str = "localhost";
const PORT: u16 = 15657;

impl ElevIo {
    pub fn new() -> io::Result<Self> {
        let (to_elev_sender, to_elev_reciver) = channel::<std::vec::Vec<u8>>();
        let (from_elev_sender, from_elev_reciver) = channel::<std::vec::Vec<u8>>();
        let elev = ElevIo { io: Communication::new(String::from(IP_ADDRESS), PORT, to_elev_reciver, from_elev_sender)?, to_elevator: to_elev_sender};
        elev.set_all_light(Light::Off)?;
        elev.set_floor_light(Floor::At(0))?;
        thread::spawn(move || {
            loop {
                match from_elev_reciver.recv() {
                    Ok(value) => {
                        println!("Got something from elevator: {:?}", value);
                    },
                    Err(_) => {
                        println!("Recv Error");
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
                self.to_elevator.send(dir);
            },
            MotorDir::Up => {
                let dir = vec![1, 1, 0, 0];
                self.to_elevator.send(dir);
            },
            MotorDir::Down => {
                let dir = vec![1, 255, 0, 0];
                self.to_elevator.send(dir);
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
        self.to_elevator.send(light_command);
        Ok(())
        
    }

    pub fn set_floor_light(&self, floor: Floor) -> io::Result<()> {
        if let Floor::At(etg) = floor {
            if etg > TOP {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "given floor is not supported"));
            }
            let mut floor_vec = vec![3, etg, 0, 0];
            self.to_elevator.send(floor_vec);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot set light between floors"))
        }
    }

    pub fn set_door_light(&self, mode: Light) -> io::Result<()> {
        match mode {
            Light::On => {
                self.to_elevator.send(vec![4, 1, 0, 0]);
            },
            Light::Off => {
                self.to_elevator.send(vec![4, 0, 0, 0]);
            },
        }
        Ok(())
    }

    pub fn set_stop_light(&self, mode: Light) -> io::Result<()> {
        match mode {
            Light::On => {
                self.to_elevator.send(vec![5, 1, 0, 0]);
            },
            Light::Off => {
                self.to_elevator.send(vec![5, 0, 0, 0]);
            },
        }
        Ok(())
    }
    /* Get functions are commented out for inital testing
    pub fn get_button_signal(&self, button: Button) -> io::Result<Signal> {
        const CALL_UP_ADDR: [usize; 3] = [ 0x300+17, 0x300+16, 0x200+1 ];
        const CALL_DOWN_ADDR: [usize; 3] = [ 0x200+0, 0x200+2, 0x200+3 ];
        const INTERNAL_ADDR: [usize; 4] = [ 0x300+21, 0x300+20, 0x300+19, 0x300+18 ];
        let addr = match button {
            Button::CallUp(Floor::At(floor @ 0...SEC_TOP)) => CALL_UP_ADDR[floor],
            Button::CallDown(Floor::At(floor @ 1...TOP)) => CALL_DOWN_ADDR[floor-1],
            Button::Internal(Floor::At(floor @ 0...TOP)) => INTERNAL_ADDR[floor],
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "given floor is not supported for given button")),
        };
        let value = self.io.read_bit(addr)?;
        Ok(Signal::new(value))
    }

    pub fn get_floor_signal(&self) -> io::Result<Floor> {
        const FLOOR_SENSOR_ADDR: [usize; 4] = [ 0x200+4, 0x200+5, 0x200+6, 0x200+7 ];
        for (floor, addr) in FLOOR_SENSOR_ADDR.iter().enumerate() {
            if self.io.read_bit(*addr)? != 0 {
                return Ok(Floor::At(floor));
            }
        }
        Ok(Floor::Between) 
    }

    pub fn get_stop_signal(&self) -> io::Result<Signal> {
        const STOP_SENSOR_ADDR: usize = 0x300+22;
        Ok(Signal::new(self.io.read_bit(STOP_SENSOR_ADDR)?))
    }

    pub fn get_obstr_signal(&self) -> io::Result<Signal> {
        const OBSTR_SENSOR_ADDR: usize = 0x300+23;
        Ok(Signal::new(self.io.read_bit(OBSTR_SENSOR_ADDR)?))
    }
    */
}
