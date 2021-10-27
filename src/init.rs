use bootloader::boot_info::FrameBuffer;

use crate::gdt;
use crate::interrupts;
use crate::log::LockedLogger;
use crate::print;
use crate::println;

pub fn init(sysframe: &'static mut FrameBuffer) {
	let info = sysframe.info();
	let framebuffer = sysframe.buffer_mut();

	LockedLogger::init(framebuffer, info);
	println!("Logger setup done.");

	print!("Loading interrupts and GDT...");
	interrupts::init_idt();
	gdt::init();
	println!(" done.");
}
