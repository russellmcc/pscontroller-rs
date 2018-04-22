#![feature(duration_from_micros)]

extern crate linux_embedded_hal as linux_hal;
extern crate embedded_hal;
extern crate pscontroller_rs;

use std::io;
use std::{thread, time};
use linux_hal::Spidev;
use linux_hal::spidev::{SpidevOptions, SPI_MODE_3};
use linux_hal::Pin;

use pscontroller_rs::PlayStationPort;

// Specific to the host device used on Linux, you'll have to change the following
// parameters depending on your board and also export and allow writing to the GPIO
const SPI_ENABLE_PIN: u64 = 1020; // XIO-7 on the NTC CHIP
const SPI_DEVICE: &str = "/dev/spidev32766.0"; // Needs a device-tree overlay
const SPI_SPEED: u32 = 10_000; // Due to a bug, 50KHz is the best the CHIP can do semi-reliably

fn build_spi() -> io::Result<Spidev> {
	let mut spi = Spidev::open(SPI_DEVICE)?;
	let opts = SpidevOptions::new()
		.bits_per_word(8)
		.max_speed_hz(SPI_SPEED)
		.mode(SPI_MODE_3)
		.build();
	spi.configure(&opts)?;

	Ok(spi)
}

fn main() {
    let spi = build_spi().unwrap();
    let enable_pin = Pin::new(SPI_ENABLE_PIN);    
    let mut psp = PlayStationPort::new(spi, enable_pin);
	let mut command = [0u8; 21];
	let mut buffer = [0u8; 21];

	command[0] = 0x01;
	command[1] = 0x42;
	command[2] = 0x00;

	let mut now = time::Instant::now();
	let sleep_duration = time::Duration::from_micros(30_000);
	let sample_duration = time::Duration::from_secs(1);
	let mut count = 0;
	let mut failure = 0;
	let mut rate = String::new();

	loop {
		thread::sleep(sleep_duration);	

		psp.send_command(&command, &mut buffer).unwrap()
		;

		if now.elapsed() > sample_duration {
			now = time::Instant::now();
			rate = format!("{0:04}/{1:04}", count, failure);
			count = 0;
			failure = 0;
		}
		println!("");
		print!("Rate: ({}) - ", rate);

		for item in buffer.iter() {
			print!("{:02x} ", item);
		}

		if buffer[1] == 0xff {
			failure += 1;
		} else {
			count += 1;
		}
	}
}
