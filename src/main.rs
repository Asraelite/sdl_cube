#![allow(unused)]

mod geometry;
mod window;
mod world;

use geometry::{Scalar, Vector3};
use window::{InputState, Window};
use world::World;

pub use window::external_exports::*;
pub use window::backend;

#[macro_use]
extern crate lazy_static;

fn main() {
	window::begin_loop(
		Window::new(),
		GameState::new(),
		move |window: &mut Window, game_state: &mut GameState| {
			window.tick(game_state);
			window.render(game_state);
		},
	);
}

mod prelude {
	pub fn elog<T: std::borrow::Borrow<str> + std::fmt::Display>(msg: T) {
		super::backend::print(msg.borrow());
	}

	pub fn log<T: std::borrow::Borrow<str> + std::fmt::Display>(msg: T) {
		super::backend::print(msg.borrow());
	}
}

pub struct GameState {
	world: World,
}

impl GameState {
	pub fn new() -> Self {
		Self {
			world: World::new(),
		}
	}

	pub fn tick(&mut self, input_state: &InputState) {
		self.world.tick(input_state);
	}
}
