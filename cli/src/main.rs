mod tm2;
mod shader;
mod texture;
mod vbo;
mod vertex;

use tim2;
use tim2::Error;
use shader::Shader;
use std::cell::Cell;
use std::sync::mpsc::Receiver;
use std::vec;
use texture::Texture;
use vbo::VBO;
use vertex::TextureVertex;

use glfw::{
	Action,
	Context,
	Key,
	Glfw,
	Window,
	WindowEvent,
	WindowHint,
	WindowMode,
};

const SRC_VERTEX: &str = r#"
	#version 330 core

	layout (location = 0) in vec2 a_pos;
	layout (location = 1) in vec2 a_coord;

	out vec2 v_coord;

	void main() {
		v_coord = a_coord;
		gl_Position = vec4(a_pos.x, a_pos.y, 0.0, 1.0);
	}
"#;

const SRC_FRAGMENT: &str = r#"
  #version 330 core

  uniform sampler2D u_tex;

  in vec2 v_coord;

  out vec4 out_color;

  void main() {
	out_color = texture(u_tex, v_coord);
  }
"#;

fn draw(shader: &Shader, texture: &Texture, vbo: &VBO) {
	shader.bind();
	texture.bind(0);
	vbo.draw();
}

fn init_glfw() -> Glfw {
	let mut glfw = glfw::init(Some(glfw::Callback {
		f: error_callback,
		data: Cell::new(0),
	})).unwrap();

	glfw.window_hint(WindowHint::ContextVersion(3, 3));
	glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
	glfw.window_hint(WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

	glfw
}

fn init_window(glfw: &Glfw) -> (Window, Receiver<(f64, WindowEvent)>) {
	let (mut window, events) = glfw.create_window(
		128,
		128,
		"TM2 Viewer",
		WindowMode::Windowed,
	).expect("Failed to create GLFW window.");

	window.make_current();
	window.set_key_polling(true);
	window.set_framebuffer_size_polling(true);

	(window, events)
}

fn init_gl(window: &mut Window) {
	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

	unsafe {
		gl::Enable(gl::BLEND);
		gl::ClearColor(0.2, 0.3, 0.3, 1.0);
		gl::ActiveTexture(gl::TEXTURE0);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}
}

fn error_callback(_: glfw::Error, description: String, error_count: &Cell<usize>) {
	println!("GLFW error ({}): {}", error_count.get(), description);
	error_count.set(error_count.get() + 1);
}

fn process_events(window: &mut Window, events: &Receiver<(f64, WindowEvent)>) {
	for (_, event) in glfw::flush_messages(&events) {
		match event {
			WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
				window.set_should_close(true)
			},
			WindowEvent::FramebufferSize(width, height) => {
				unsafe {
					gl::Viewport(0, 0, width, height);
				}
			},
			_ => {},
		}
	}
}

fn process_frame() {
	unsafe {
		gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
	}
}

fn main() {
	let mut glfw = init_glfw();
	let (mut window, events) = init_window(&glfw);

	init_gl(&mut window);

	let shader = Shader::make(SRC_VERTEX, SRC_FRAGMENT).unwrap();
	let vbo = VBO::make(&vec![
		TextureVertex::make( 1.0,  1.0, 1.0, 0.0),
		TextureVertex::make(-1.0,  1.0, 0.0, 0.0),
		TextureVertex::make(-1.0, -1.0, 0.0, 1.0),
		TextureVertex::make( 1.0, -1.0, 1.0, 1.0),
	], &TextureVertex::attrs());

	tm2::load("./assets/ms_000.tm2").unwrap();

	let image = tim2::load("./assets/ms_000.tm2").unwrap();
	let frame = image.get_frame(0);
	let texture = Texture::from_frame(frame, false);

	window.set_size(frame.width() as i32, frame.height() as i32);
	while !window.should_close() {
		process_events(&mut window, &events);
		process_frame();

		draw(&shader, &texture, &vbo);

		window.swap_buffers();
		glfw.poll_events();
	}
}
