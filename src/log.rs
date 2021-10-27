use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use core::{fmt, ptr};
use font8x8::UnicodeFonts;
use spinning_top::{lock_api::MutexGuard, RawSpinlock, Spinlock};

/// Additional vertical space between lines
const LINE_SPACING: usize = 4;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => ($crate::log::_print(format_args!($($arg)*)));
}

#[macro_export]
#[macro_export]
macro_rules! println {
	() => (crate::print!("\n"));
	($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)));
}

pub struct LockedLogger(pub Spinlock<Logger>);

impl LockedLogger {
	/// Create a new instance that logs to the given framebuffer and sets it to LOGGER.
	pub fn init(framebuffer: &'static mut [u8], info: FrameBufferInfo) {
		LOGGER.init_once(|| LockedLogger(Spinlock::new(Logger::new(framebuffer, info))))
	}
}

/// Allows logging text to a pixel-based framebuffer.
pub struct Logger {
	framebuffer: &'static mut [u8],
	info: FrameBufferInfo,
	x_pos: usize,
	y_pos: usize
}

impl Logger {
	/// Creates a new logger that uses the given framebuffer.
	pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
		let mut logger = Self {
			framebuffer,
			info,
			x_pos: 0,
			y_pos: 0
		};

		logger.clear();
		logger
	}

	fn newline(&mut self) {
		self.y_pos += 8 + LINE_SPACING;
		self.carriage_return()
	}

	fn carriage_return(&mut self) {
		self.x_pos = 0;
	}

	/// Erases all text on the screen.
	pub fn clear(&mut self) {
		self.x_pos = 0;
		self.y_pos = 0;
		self.framebuffer.fill(0);
	}

	fn width(&self) -> usize {
		self.info.horizontal_resolution
	}

	fn height(&self) -> usize {
		self.info.vertical_resolution
	}

	fn write_rendered_char(&mut self, rendered_char: [u8; 8]) {
		for (y, byte) in rendered_char.iter().enumerate() {
			for (x, bit) in (0..8).enumerate() {
				let alpha = if *byte & (1 << bit) == 0 { 0 } else { 255 };
				self.write_pixel(self.x_pos + x, self.y_pos + y, alpha);
			}
		}

		self.x_pos += 8;
	}

	fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
		let pixel_offset = y * self.info.stride + x;
		let color = match self.info.pixel_format {
			PixelFormat::RGB => [intensity, intensity, intensity, 0],
			PixelFormat::BGR => [intensity, intensity, intensity, 0],
			PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
			_ => unreachable!()
		};

		let bytes_per_pixel = self.info.bytes_per_pixel;
		let byte_offset = pixel_offset * bytes_per_pixel;
		self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
			.copy_from_slice(&color[..bytes_per_pixel]);

		let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
	}
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl fmt::Write for Logger {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for c in s.chars() {
			self.write_char(c)?;
		}

		Ok(())
	}

	fn write_char(&mut self, c: char) -> fmt::Result {
		match c {
			'\n' => self.newline(),
			'\r' => self.carriage_return(),
			c => {
				if self.x_pos >= self.width() {
					self.newline();
				}
				if self.y_pos >= (self.height() - 8) {
					self.clear();
				}
				let rendered = font8x8::BASIC_FONTS
					.get(c)
					.expect("character not found in basic font");

				self.write_rendered_char(rendered);
			}
		};

		Ok(())
	}
}

pub fn _print(args: fmt::Arguments<'_>) {
	use fmt::Write;

	write!(get_logger().unwrap(), "{}", args).unwrap()
}

pub fn get_logger() -> Option<MutexGuard<'static, RawSpinlock, Logger>> {
	LOGGER.get().map(|x| x.0.lock())
}
