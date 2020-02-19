# TTK4145 Rust TCP elevator driver
A tcp rust implementation of the driver used to interface with the elevator.

## Known bugs
* Delays of at least 10ms are required between commands due to channel bugs

## Credits
Based on [edvardsp](https://github.com/edvardsp)'s implementation [driver-rust](https://github.com/edvardsp/driver-rust) for direct hardware access .