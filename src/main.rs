#![allow(unused)]

mod geometry;
mod window;
mod world;

use geometry::{Scalar, Vector3};
use window::{Window, InputState};
use world::World;

fn main() {
	let mut window = Window::new();
	let mut game_state = GameState::new();

	'main: loop {
		window.tick(&mut game_state);
		window.render(&mut game_state);

		if window.should_exit {
			break 'main;
		}
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
