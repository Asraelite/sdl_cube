use std::convert::From;

use sdl2::keyboard::Keycode as SdlKeycode;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Point as SdlPoint;
use sdl2::render::Canvas;

use super::super::{Color, Keycode, WindowEvent};

pub struct Backend {
	sdl: sdl2::Sdl,
	canvas: Canvas<sdl2::video::Window>,
}

macro_rules! match_keycodes {
	(__parse $var:ident [$($path:pat => $out:expr)*]) => {
		match $var {
			$($path => $out),*
		}
	};

	(__parse $var:ident [$($parsed:tt)*] ...($($key:tt),*), $($body:tt)*) => {
		match_keycodes!(__parse $var [$($parsed)*
			$(SdlKeycode::$key => Keycode::$key)*] $($body)*)
	};

	(__parse $var:ident [$($parsed:tt)*] $a:pat => $b:expr, $($body:tt)*) => {
		match_keycodes!(__parse $var [$($parsed)* $a => $b] $($body)*)
	};

	($var:ident { $($body:tt)* }) => {
		match_keycodes!(__parse $var [] $($body)*)
	};
}

impl From<SdlKeycode> for Keycode {
	fn from(sdl_keycode: SdlKeycode) -> Keycode {
		match_keycodes!(sdl_keycode {
			...(W, S, A, D, Q, E),
			_ => Keycode::Unknown,
		})
	}
}

impl From<Color> for SdlColor {
	fn from(color: Color) -> SdlColor {
		SdlColor::RGB(color.r, color.g, color.b)
	}
}

impl Backend {
	pub fn new() -> Self {
		let sdl = sdl2::init().unwrap();
		let video_subsystem = sdl.video().unwrap();
		let window = video_subsystem
			.window("cube", 900, 700)
			.resizable()
			.build()
			.unwrap();
		let mut canvas = window.into_canvas().present_vsync().build().unwrap();

		Self { sdl, canvas }
	}

	pub fn viewport_width(&self) -> u32 {
		self.canvas.viewport().width()
	}

	pub fn viewport_height(&self) -> u32 {
		self.canvas.viewport().height()
	}

	pub fn clear_canvas(&mut self) {
		self.canvas.set_draw_color(SdlColor::BLACK);
		self.canvas.clear();
	}

	pub fn update_canvas(&mut self) {
		self.canvas.present();
	}

	pub fn set_draw_color(&mut self, color: Color) {
		let sdl_color = self.canvas.set_draw_color(color);
	}

	pub fn draw_line(&mut self, start: (f32, f32), end: (f32, f32)) {
		self.draw_lines(&[start, end]);
	}

	pub fn draw_lines(&mut self, lines: &[(f32, f32)]) {
		let lines: Vec<SdlPoint> = lines
			.iter()
			.map(|&(x, y)| (x as i32, y as i32).into())
			.collect();

		self.canvas.draw_lines(lines.as_slice());
	}

	pub fn poll_event(&mut self) -> Option<WindowEvent> {
		let sdl_event = self.sdl.event_pump().unwrap().poll_event();
		if (sdl_event.is_none()) {
			return None;
		}

		use sdl2::event::Event as S;
		use WindowEvent as W;
		Some(match sdl_event.unwrap() {
			S::Quit { .. } => W::Quit,
			S::KeyDown {
				keycode: Some(keycode),
				..
			} => W::KeyDown(keycode.into()),
			S::KeyUp {
				keycode: Some(keycode),
				..
			} => W::KeyUp(keycode.into()),
			_ => return None,
		})
	}
}
