use crate::prelude::*;

use super::super::super::GameState;
use super::super::{Color, Keycode, Window, WindowEvent};

use std::sync::Mutex;
use std::collections::VecDeque;

extern "C" {
	fn console_log(ptr: *const u8, len: u32);

	fn canvas_set_stroke_color(r: u8, g: u8, b: u8);
	fn canvas_stroke();
	fn canvas_begin_path();
	fn canvas_move_to(x: f64, y: f64);
	fn canvas_line_to(x: f64, y: f64);
	fn canvas_clear();
	fn canvas_width() -> u32;
	fn canvas_height() -> u32;

	fn random() -> f64;
}

fn js_log<T: std::borrow::Borrow<str>>(message: T) {
	let slice = message.borrow().as_bytes();
	unsafe {
		console_log(slice.as_ptr(), slice.len() as u32);
	}
}

type LoopClosure =
	Mutex<Option<Box<dyn Fn(&mut Window, &mut GameState) + Send>>>;

lazy_static! {
	static ref LOOPING_WINDOW: Mutex<Option<Window>> = Mutex::new(None);
	static ref LOOPING_GAME_STATE: Mutex<Option<GameState>> = Mutex::new(None);
	static ref LOOPING_CLOSURE: LoopClosure = Mutex::new(None);
	static ref EVENTS: Mutex<VecDeque<WindowEvent>> = Mutex::new(VecDeque::new());
}

pub mod external_exports {
	use super::*;

	#[no_mangle]
	pub fn tick() {
		let mut window = super::LOOPING_WINDOW.lock().unwrap();
		let mut game_state = super::LOOPING_GAME_STATE.lock().unwrap();
		let mut closure = super::LOOPING_CLOSURE.lock().unwrap();

		closure.as_ref().unwrap()(
			window.as_mut().unwrap(),
			game_state.as_mut().unwrap(),
		);
	}

	#[no_mangle]
	pub fn key_down_event(keycode: i32) {
		queue_event(WindowEvent::KeyDown(super::match_keycode_num(keycode)));
	}

	#[no_mangle]
	pub fn key_up_event(keycode: i32) {
		queue_event(WindowEvent::KeyUp(super::match_keycode_num(keycode)));
	}
}

fn queue_event(event: WindowEvent) {
	EVENTS.lock().unwrap().push_back(event);
}

pub mod random {
	pub fn rangei(start: isize, end: isize) -> isize {
		let r = unsafe { super::random() };
		(r * (end as f64 - start as f64) + start as f64).floor() as isize
	}
}

pub fn print(msg: &str) {
	js_log(msg);
}

fn set_panic_hook() {
	std::panic::set_hook(Box::new(|panic_info| {
		let payload = panic_info.payload();

		let message = if let Some(message) = payload.downcast_ref::<String>() {
			message.as_str()
		} else if let Some(message) = payload.downcast_ref::<&str>() {
			message
		} else {
			"Unknown panic payload type"
		};

		let location_string = if let Some(location) = panic_info.location() {
			format!("{} {}", location.file(), location.line())
		} else {
			String::from("Unknown location")
		};

		js_log(format!("Panic: {:?}\n\tat {}\n", message, location_string));
	}));
}

pub fn begin_loop(
	mut window: Window,
	mut game_state: GameState,
	closure: impl Fn(&mut Window, &mut GameState) + Send + 'static,
) {
	*LOOPING_WINDOW.lock().unwrap() = Some(window);
	*LOOPING_GAME_STATE.lock().unwrap() = Some(game_state);
	*LOOPING_CLOSURE.lock().unwrap() = Some(Box::new(closure));
	//self.backend.begin_loop(closure);
}

pub struct Backend {}

impl Backend {
	pub fn new() -> Self {
		set_panic_hook();

		Self {}
	}
	// TODO
	pub fn viewport_width(&self) -> u32 {
		unsafe { canvas_width() }
	}

	pub fn viewport_height(&self) -> u32 {
		unsafe { canvas_height() }
	}

	pub fn clear_canvas(&mut self) {
		unsafe { canvas_clear() }
	}

	pub fn update_canvas(&mut self) {}

	pub fn set_draw_color(&mut self, color: Color) {
		unsafe { canvas_set_stroke_color(color.r, color.g, color.b) }
	}

	pub fn draw_line(&mut self, start: (f32, f32), end: (f32, f32)) {
		self.draw_lines(&[start, end]);
	}

	pub fn draw_lines(&mut self, lines: &[(f32, f32)]) {
		if lines.len() == 0 {
			return;
		}

		unsafe {
			canvas_begin_path();
			canvas_move_to(lines[0].0 as f64, lines[0].1 as f64);
		}

		for &(x, y) in &lines[1..] {
			unsafe { canvas_line_to(x as f64, y as f64) };
		}

		unsafe { canvas_stroke() };

		//self.canvas.draw_lines(lines.as_slice());
	}

	pub fn poll_event(&mut self) -> Option<WindowEvent> {
		EVENTS.lock().unwrap().pop_front()
	}
}

fn match_keycode_num(num: i32) -> Keycode {
	match num {
		0 => Keycode::A,
		3 => Keycode::D,
		4 => Keycode::E,
		16 => Keycode::Q,
		18 => Keycode::S,
		22 => Keycode::W,
		100 => Keycode::Escape,
		_ => Keycode::Unknown,
	}
}
