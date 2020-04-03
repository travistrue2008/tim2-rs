use crate::vbo::AttributeKind;

use std::vec::Vec;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct TextureVertex {
	pub x: f32,
	pub y: f32,
	pub u: f32,
	pub v: f32,
}

impl TextureVertex {
	pub fn attrs() -> Vec<(bool, usize, AttributeKind)> {
		vec![
			(false, 2, AttributeKind::Float),
			(false, 2, AttributeKind::Float),
		]
	}

	pub fn new() -> TextureVertex {
		TextureVertex { x: 0.0, y: 0.0, u: 0.0, v: 0.0 }
	}

	pub fn make(x: f32, y: f32, u: f32, v: f32) -> TextureVertex {
		TextureVertex { x, y, u, v }
	}
}
