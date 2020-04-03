use crate::error::Error;

use gl::types::*;
use std::os::raw::c_void;
use tim2::Frame;
use tim2::Pixel;

pub enum MinFilter {
	Nearest,
	Linear,
	NearestMipmapNearest,
	NearestMipmapLinear,
	LinearMipmapNearest,
	LinearMipmapLinear,
}

impl MinFilter {
	pub fn get_native(&self) -> GLenum {
		match self {
			MinFilter::Nearest => gl::NEAREST,
			MinFilter::Linear => gl::LINEAR,
			MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
			MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
			MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_LINEAR,
			MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
		}
	}
}

pub enum MagFilter {
	Nearest,
	Linear,
}

impl MagFilter {
	pub fn get_native(&self) -> GLenum {
		match self {
			MagFilter::Nearest => gl::NEAREST,
			MagFilter::Linear => gl::LINEAR,
		}
	}
}

pub struct Texture {
	mipmaps: bool,
	handle: GLuint,
	minFilter: MinFilter,
	magFilter: MagFilter,
}

impl Texture {
	pub fn from_frame(frame: &Frame, mipmaps: bool) -> Texture {
		let mut handle = 0 as GLuint;
		let color_key = Pixel::from(0, 255, 0, 255);
		let raw = frame.to_raw(Some(color_key));
	
		unsafe {
			gl::GenTextures(1, &mut handle);
			gl::BindTexture(gl::TEXTURE_2D, handle);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
	
			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				gl::RGBA as i32,
				frame.width() as i32,
				frame.height() as i32,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				&raw[0] as *const u8 as *const c_void,
			);

			if mipmaps {
				gl::GenerateMipmap(gl::TEXTURE_2D);
			}

			Texture {
				mipmaps,
				handle,
				minFilter: MinFilter::Nearest,
				magFilter: MagFilter::Nearest,
			}
		}
	}

	pub fn bind(&self, unit: GLenum) {
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 + unit);
			gl::BindTexture(gl::TEXTURE_2D, self.handle);
		}
	}

	pub fn set_min_filter(&mut self, filter: MinFilter) -> Result<(), Error> {
		self.bind(0);

		match filter {
			MinFilter::Nearest | MinFilter::Linear => (),
			MinFilter::NearestMipmapNearest |
			MinFilter::NearestMipmapLinear |
			MinFilter::LinearMipmapNearest |
			MinFilter::LinearMipmapLinear => {
				if !self.mipmaps {
					return Err(Error::NoMipmaps);
				}
			},
		};

		unsafe {
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter.get_native() as i32);
		}

		self.minFilter = filter;
		Ok(())
	}

	pub fn set_mag_filter(&mut self, filter: MagFilter) {
		self.bind(0);

		unsafe {
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter.get_native() as i32);
		}

		self.magFilter = filter;
	}
}

impl Drop for Texture {
	fn drop(&mut self) {
		unsafe { gl::DeleteTextures(1, &self.handle) };
		self.handle = 0;
	}
}
