use crate::common::*;
use crate::error::Error;
use crate::pixel::{Format, Pixel};

use byteorder::{ByteOrder, LittleEndian};

const SWIZZLE_WIDTH: usize = 16;
const SWIZZLE_HEIGHT: usize = 8;

pub type PixelBuffer = Vec::<Pixel>;

#[derive(Debug)]
pub enum DataKind {
	Indices(Vec::<u8>),
	Pixels(PixelBuffer),
}

impl DataKind {
	pub fn len(&self) -> usize {
		match self {
			DataKind::Indices(v) => v.len(),
			DataKind::Pixels(v) => v.len(),
		}
	}
}

#[derive(Debug)]
struct Header {
	total_size: u32,
    palette_size: u32,
    image_size: u32,
    header_size: u16,
    color_entry_count: u16,
    paletted: u8,
	mipmap_count: u8,
	clut_format: u8,
	bpp: u8,
	width: usize,
	height: usize,
	gs_regs: u32,
	gs_tex_clut: u32,
	gs_tex_0: u64,
	gs_tex_1: u64,
	user_data: Vec::<u8>,
}

impl Header {
	pub fn read(buffer: &[u8], offset: &mut usize) -> Result<Header, Error> {
		let mut load_part = |size| { get_slice(&buffer, offset, size) };

		let mut result = Header {
			total_size: LittleEndian::read_u32(load_part(4)),
			palette_size: LittleEndian::read_u32(load_part(4)),
			image_size: LittleEndian::read_u32(load_part(4)),
			header_size: LittleEndian::read_u16(load_part(2)),
			color_entry_count: LittleEndian::read_u16(load_part(2)),
			paletted: load_part(1)[0],
			mipmap_count: load_part(1)[0],
			clut_format: load_part(1)[0],
			bpp: Header::find_bpp(load_part(1)[0])?,
			width: LittleEndian::read_u16(load_part(2)) as usize,
			height: LittleEndian::read_u16(load_part(2)) as usize,
			gs_tex_0: LittleEndian::read_u64(load_part(8)),
			gs_tex_1: LittleEndian::read_u64(load_part(8)),
			gs_regs: LittleEndian::read_u32(load_part(4)),
			gs_tex_clut: LittleEndian::read_u32(load_part(4)),
			user_data: Vec::new(),
		};

		let user_data_size = result.header_size as usize - 48;
		if user_data_size > 0 {
			result.user_data = load_part(user_data_size).to_vec();
		}

		if result.palette_size > 0 && result.bpp > 8 {
			Err(Error::TrueColorAndPaletteFound)
		} else {
			Ok(result)
		}
	}

	fn find_bpp(v: u8) -> Result<u8, Error> {
		match v {
			1 => Ok(16),
			2 => Ok(24),
			3 => Ok(32),
			4 => Ok(4),
			5 => Ok(8),
			n => Err(Error::InvalidBppFormat(n)),
		}
	}

	pub fn is_linear_palette(&self) -> bool {
		self.clut_format & 0x80 != 0
	}

	pub fn color_size(&self) -> usize {
		if self.bpp > 8 {
			(self.bpp / 8) as usize
		} else {
			(self.clut_format & 0x07) as usize + 1
		}
	}

	pub fn pixel_format(&self) -> Result<Format, Error> {
		match self.bpp {
			4 => Ok(Format::Indexed4),
			8 => Ok(Format::Indexed8),
			16 => Ok(Format::Abgr1555),
			24 => Ok(Format::Rgb888),
			32 => Ok(Format::Rgba8888),
			n => Err(Error::InvalidBpp(n)),
		}
	}
}

#[derive(Debug)]
pub struct Frame {
	header: Header,
	data: DataKind,
	palettes: Vec::<PixelBuffer>,
}

impl Frame {
	pub fn read(buffer: &[u8], offset: &mut usize) -> Result<Frame, Error> {
		let header = Header::read(buffer, offset)?;
		let data = Frame::read_data(buffer, offset, &header)?;
		let palettes= Frame::read_palettes(buffer, offset, &header)?;

		Ok(Frame { header, data, palettes })
	}

	fn read_data(buffer: &[u8], offset: &mut usize, header: &Header) -> Result<DataKind, Error> {
		let pixel_size = header.bpp as usize / 8;
		let size = header.image_size as usize;
		let slice = get_slice(buffer, offset, size);
		let data = if header.bpp == 4 {
			let mut result = Vec::with_capacity(slice.len() * 2);

			for index_pair in slice {
				result.push(*index_pair & 0xF0 >> 4);
				result.push(*index_pair & 0xF);
			}

			result
		} else {
			slice.to_vec()
		};

		if header.palette_size > 0 {
			let raw = if header.gs_tex_0 & (1 << 55) != 0 {
				Frame::unswizzle(&data, header)
			} else {
				data
			};

			Ok(DataKind::Indices(raw))
		} else {
			let colors = Frame::read_colors(&data, pixel_size)?;
			let raw = if header.gs_tex_0 & (1 << 55) != 0 {
				Frame::unswizzle(&colors, header)
			} else {
				colors
			};

			Ok(DataKind::Pixels(raw))
		}
	}

	fn read_palettes(buffer: &[u8], offset: &mut usize, header: &Header) -> Result<Vec<PixelBuffer>, Error> {
		if header.palette_size == 0 {
			return Ok(Vec::new());
		}

		let total_size = header.palette_size as usize;
		let slice = get_slice(buffer, offset, total_size);
		let size = header.color_entry_count as usize * header.color_size();
		let count = total_size / size;
		let color_size = header.color_size();
		let mut result = Vec::with_capacity(count);

		for i in 0..count {
			let start_index = size * i;
			let end_index = start_index + size;
			let data = &slice[start_index..end_index];
			let mut palette = Frame::read_colors(data, color_size)?;

			if !header.is_linear_palette() && header.bpp == 8 {
				Frame::linearize_palette(&mut palette);
			}

			result.push(palette);
		}

		Ok(result)
	}

	fn read_colors(buffer: &[u8], color_size: usize) -> Result<PixelBuffer, Error> {
		let mut offset = 0usize;
		let mut result = Vec::new();

		for _ in (0..buffer.len()).step_by(color_size) {
			let slice = get_slice(buffer, &mut offset, color_size);
			let pixel = Pixel::from_buf(slice)?;

			result.push(pixel)
		}

		Ok(result)
	}

	fn linearize_palette(palette: &mut PixelBuffer) {
		const COLOR_COUNT: usize = 8;
		const BLOCK_COUNT: usize = 2;
		const STRIPE_COUNT: usize = 2;

		let mut i = 0usize;
		let part_count = palette.len() / 32;
		let original = palette.clone();

		for part in 0..part_count {
			for block in 0..BLOCK_COUNT {
				for stripe in 0..STRIPE_COUNT {
					for color in 0..COLOR_COUNT {
						let i1 = part * COLOR_COUNT * STRIPE_COUNT * BLOCK_COUNT;
						let i2 = block * COLOR_COUNT;
						let i3 = stripe * STRIPE_COUNT * COLOR_COUNT;

						palette[i] = original[i1 + i2 + i3 + color];
						i += 1;
					}
				}
			}
		}
	}

	fn unswizzle<T: Default + Copy>(buffer: &Vec::<T>, header: &Header) -> Vec::<T> {
		let mut i = 0usize;
		let mut result = vec![Default::default(); buffer.len()];

		for y in (0..header.height).step_by(SWIZZLE_HEIGHT) {
			for x in (0..header.width).step_by(SWIZZLE_WIDTH) {
				for tile_y in y..(y + SWIZZLE_HEIGHT) {
					for tile_x in x..(x + SWIZZLE_WIDTH) {
						if tile_x < header.width && tile_y < header.height {
							let index = tile_y * header.width + tile_x;

							if let Some(value) = buffer.get(i) {
								result[index] = *value;
							}
						}

						i += 1;
					}
				}
			}
		}

		result
	}

	pub fn has_mipmaps(&self) -> bool {
		self.header.mipmap_count > 1
	}

	pub fn width(&self) -> usize {
		self.header.width as usize
	}

	pub fn height(&self) -> usize {
		self.header.height as usize
	}

	pub fn format(&self) -> Result<Format, Error> {
		self.header.pixel_format()
	}

	pub fn data(&self) -> &DataKind {
		&self.data
	}

	pub fn get_pixels(&self) -> PixelBuffer {
		match &self.data {
			DataKind::Indices(v) => {
				let palette = &self.palettes[0];
				let mut result = Vec::with_capacity(v.len());

				for index in v {
					result.push(palette[*index as usize]);
				}

				result
			},
			DataKind::Pixels(v) => v.to_vec(),
		}
	}

	pub fn to_raw(&self, color_key: Option<Pixel>) -> Vec::<u8> {
		let pixels = self.get_pixels();
		let mut result = Vec::with_capacity(pixels.len() * 4);

		for pixel in pixels {
			let alpha = if let Some(v) = color_key {
				if v != pixel { pixel.a() } else { 0 }
			} else {
				pixel.a()
			};

			result.push(pixel.r());
			result.push(pixel.g());
			result.push(pixel.b());
			result.push(alpha);
		}

		result
	}
}
