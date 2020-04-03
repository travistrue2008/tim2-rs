use crate::common::*;
use crate::error::Error;
use crate::frame::Frame;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

const IDENT: u32 = 0x54494d32;

#[derive(Debug)]
struct Header {
	identifier: u32,
	version: u16,
	count: usize,
}

impl Header {
	fn read(buffer: &[u8], offset: &mut usize) -> Result<Header, Error> {
		let mut load_part = |size| { get_slice(&buffer, offset, size) };
		let identifier = BigEndian::read_u32(load_part(4));
		let version = LittleEndian::read_u16(load_part(2));
		let count = LittleEndian::read_u16(load_part(2)) as usize;

		load_part(8);
		if identifier != IDENT {
			return Err(Error::InvalidIdentifier(identifier))
		}

		Ok(Header { identifier, version, count })
	}
}

#[derive(Debug)]
pub struct Image {
	header: Header,
	frames: Vec::<Frame>,
}

impl Image {
	fn read(buffer: &[u8], offset: &mut usize) -> Result<Image, Error> {
		let header = Header::read(buffer, offset)?;
		let mut frames = Vec::with_capacity(header.count);

		for _ in 0..header.count {
			frames.push(Frame::read(buffer, offset)?);
		}

		Ok(Image { header, frames })
	}

	pub fn frames(&self) -> &Vec::<Frame> {
		&self.frames
	}

	pub fn get_frame(&self, index: usize) -> &Frame {
		&self.frames[index]
	}
}

/// Loads a TIM2 image file into memory.
///
/// # Examples
///
/// ```
/// fn main() {
///     let image = tim2::load("../assets/test.tm2").unwrap();
/// 
///     /* print the header info for each frame found */
///     for (i, frame) in image.frames().iter().enumerate() {
///         println!("frame[{}]: <{}  {}>", i, frame.width(), frame.height());
///     }
/// }
/// ```
pub fn load<P: AsRef<Path>>(path: P) -> Result<Image, Error> {
	let mut offset = 0usize;
	let mut buffer = Vec::new();
	let mut file = File::open(path)?;

	file.read_to_end(&mut buffer)?;
	Image::read(&buffer, &mut offset)
}
