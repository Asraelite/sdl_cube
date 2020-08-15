#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Keycode {
	A,
	D,
	E,
	Q,
	S,
	W,
	Escape,

	Unknown,
}

pub enum WindowEvent {
	KeyDown(Keycode),
	KeyUp(Keycode),
	Quit,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

macro_rules! const_color {
	($name:ident, $r:expr, $g:expr, $b:expr) => {
		pub fn $name() -> Self {
			Self::rgb($r, $g, $b)
		}
	};
}

impl Color {
	pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
		Self { r, g, b }
	}

	pub const BLUE: Self = Self::rgb(0, 0, 255);
	pub const CYAN: Self = Self::rgb(0, 255, 255);
	pub const GRAY: Self = Self::rgb(128, 128, 128);
	pub const BLACK: Self = Self::rgb(0, 0, 0);
	pub const WHITE: Self = Self::rgb(255, 255, 255);
}
