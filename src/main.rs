#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]
#![warn(clippy::all)]

mod gdt;
mod init;
mod interrupts;
mod log;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
	if let Some(sysframe) = boot_info.framebuffer.as_mut() {
		init::init(sysframe)
	} else {
		// No framebuffer found. How's this possible?
	}

	loop {
		x86_64::instructions::hlt()
	}
}

#[allow(clippy::print_literal)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);

	loop {
		x86_64::instructions::hlt()
	}
}
