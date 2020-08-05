mod projection;

use std::collections::HashSet;

pub use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;

use super::world::{Direction, Entity, Frame, World};
use super::GameState;
use crate::geometry::{vec3, Matrix4x4, Vector3, PI};

use projection::{Camera, CameraProjector};

pub struct Window {
	sdl: sdl2::Sdl,
	canvas: Canvas<sdl2::video::Window>,
	input_state: WindowInputState,
	pub should_exit: bool,
	tick: usize,
}

pub struct WindowInputState {
	// Keyboard keys that started being pressed this frame
	pub keys_pressed: HashSet<Keycode>,
	// Keyboard keys that have not yet been released, regardless of when
	// they started being pressed.
	pub keys_held: HashSet<Keycode>,
}

impl WindowInputState {
	pub fn new() -> Self {
		Self {
			keys_pressed: HashSet::new(),
			keys_held: HashSet::new(),
		}
	}

	pub fn key_down_event(&mut self, keycode: Keycode) {
		self.keys_held.insert(keycode);
		self.keys_pressed.insert(keycode);
	}

	pub fn key_up_event(&mut self, keycode: Keycode) {
		self.keys_held.remove(&keycode);
	}

	// Run at the end of every frame to ensure keys in `keys_pressed`
	// no longer count as pressed in the next frame.
	pub fn clear_frame(&mut self) {
		self.keys_pressed.clear();
	}
}

impl Window {
	pub fn new() -> Self {
		let sdl = sdl2::init().unwrap();
		let video_subsystem = sdl.video().unwrap();
		let window = video_subsystem
			.window("cube", 900, 700)
			.resizable()
			.build()
			.unwrap();
		let mut canvas = window.into_canvas().present_vsync().build().unwrap();

		Self {
			sdl,
			canvas,
			input_state: WindowInputState::new(),
			should_exit: false,
			tick: 0,
		}
	}

	pub fn tick(&mut self, game_state: &mut GameState) {
		let mut event_pump = self.sdl.event_pump().unwrap();

		for event in event_pump.poll_iter() {
			use sdl2::event::Event::*;
			match event {
				Quit { .. } => self.should_exit = true,
				KeyDown {
					keycode: Some(keycode),
					..
				} => self.input_state.key_down_event(keycode),
				KeyUp {
					keycode: Some(keycode),
					..
				} => self.input_state.key_up_event(keycode),
				_ => {}
			}
		}

		game_state.tick(&self.input_state);
		self.input_state.clear_frame();
		self.tick += 1;
	}

	pub fn render(&mut self, game_state: &mut GameState) {
		self.canvas.set_draw_color(Color::RGB(0, 0, 0));
		self.canvas.clear();

		let projector = {
			let position = Vector3::new(0.0, 0.0, 300.0);
			let rotation = Vector3::new(0.0, 0.0, 0.0);
			let fov_degrees = 50.0;
			let camera = Camera::new(position, rotation, fov_degrees);

			let viewport_rect = self.canvas.viewport();
			let viewport_width = viewport_rect.width() as f32;
			let viewport_height = viewport_rect.height() as f32;

			camera.projector(viewport_width, viewport_height)
		};

		self.render_cube(&projector, game_state);

		// let red = Color::RED;
		// let from = Vector3::new(-5.0, 20.0, 0.0);
		// let to = Vector3::new(30.0, -10.0, 5.0);
		// self.draw_line(&projector, from, to, red);

		self.canvas.present();
	}

	fn render_cube(
		&mut self,
		projector: &CameraProjector,
		game_state: &mut GameState,
	) {
		// let red = Color::RED;
		// let from = Vector3::new(-5.0, 20.0, 0.0);
		// let to = Vector3::new(30.0, -10.0, 5.0);
		// self.draw_line(&projector, from, to, red);

		let world = &game_state.world;

		let focus_entity_id = world.focus_entity.expect("No focus entity");
		let focus_entity = world.get_entity(focus_entity_id).unwrap();
		let focus_position = focus_entity.position;

		let frame = world.get_frame(focus_position.frame).unwrap();

		let focus_x = (focus_position.x).powf(2.0) * focus_position.x.signum();
		let focus_y = (focus_position.y).powf(2.0) * focus_position.y.signum();

		let r = vec3(
			focus_y * (PI / 4.0),
			focus_x * -(PI / 4.0),
			0.0,
		);

		//println!("{:?}", focus_position);

		self.draw_frame(projector, &frame, Direction::Neutral, r);
		self.draw_frame(projector, &frame, Direction::Up, r);
		self.draw_frame(projector, &frame, Direction::Down, r);
		self.draw_frame(projector, &frame, Direction::Left, r);
		self.draw_frame(projector, &frame, Direction::Right, r);

		for entity_id in world.entity_ids() {
			let entity = world.get_entity(entity_id).unwrap();
			let frame = entity.position.frame;
			self.draw_entity(projector, entity, Direction::Neutral, r);
		}
	}

	fn draw_entity(
		&mut self,
		projector: &CameraProjector,
		entity: &Entity,
		direction: Direction,
		rotation: Vector3,
	) {
		let (mut rotate_pitch, mut rotate_roll) = match direction {
			Direction::Neutral => (0.0, 0.0),
			Direction::Up => (PI / 2.0, 0.0),
			Direction::Down => (-PI / 2.0, 0.0),
			Direction::Right => (0.0, PI / 2.0),
			Direction::Left => (0.0, -PI / 2.0),
			_ => (0.0, 0.0),
		};

		//rotate_pitch += (self.tick as f32 / 100.0);

		let mut direction_rotation = Matrix4x4::identity().rotated(
			rotate_pitch,
			rotate_roll,
			0.0,
		);

		let view_rotation = Matrix4x4::identity().rotated(
			rotation.x,
			rotation.y,
			rotation.z,
		);

		let m = direction_rotation;
		let r = view_rotation;
		let p = entity.position;
		self.draw_line(
			projector,
			vec3(p.x, p.y - 0.01, 1.0) * m * r,
			vec3(p.x, p.y + 0.01, 1.0) * m * r,
			Color::CYAN,
		);
	}

	fn draw_frame(
		&mut self,
		projector: &CameraProjector,
		frame: &Frame,
		direction: Direction,
		rotation: Vector3,
	) {
		let color = Color::WHITE;

		let (mut rotate_pitch, mut rotate_roll) = match direction {
			Direction::Neutral => (0.0, 0.0),
			Direction::Up => (PI / 2.0, 0.0),
			Direction::Down => (-PI / 2.0, 0.0),
			Direction::Right => (0.0, PI / 2.0),
			Direction::Left => (0.0, -PI / 2.0),
			_ => (0.0, 0.0),
		};

		//rotate_pitch += (self.tick as f32 / 100.0);

		let mut direction_rotation = Matrix4x4::identity().rotated(
			rotate_pitch,
			rotate_roll,
			0.0,
		);

		let view_rotation = Matrix4x4::identity().rotated(
			rotation.x,
			rotation.y,
			rotation.z,
		);

		let m = direction_rotation;
		let r = view_rotation;
		self.draw_line(
			projector,
			vec3(-1.0, -1.0, 1.0) * m * r,
			vec3(1.0, -1.0, 1.0) * m * r,
			color,
		);
		self.draw_line(
			projector,
			vec3(1.0, -1.0, 1.0) * m * r,
			vec3(1.0, 1.0, 1.0) * m * r,
			color,
		);
		self.draw_line(
			projector,
			vec3(1.0, 1.0, 1.0) * m * r,
			vec3(-1.0, 1.0, 1.0) * m * r,
			color,
		);
		self.draw_line(
			projector,
			vec3(-1.0, 1.0, 1.0) * m * r,
			vec3(-1.0, -1.0, 1.0) * m * r,
			color,
		);
	}

	// fn draw_rect(&mut self, corner_a: Vector3, corner_b: Vector3, matrix: Matrix4x4) {
	// 	self.draw_line(
	// 		projector,
	// 		vec3(-1.0, -1.0, 1.0),
	// 		vec3(1.0, -1.0, 1.0),
	// 		color,
	// 	);
	// 	self.draw_line(
	// 		projector,
	// 		vec3(1.0, -1.0, 1.0),
	// 		vec3(1.0, 1.0, 1.0),
	// 		color,
	// 	);
	// 	self.draw_line(
	// 		projector,
	// 		vec3(1.0, 1.0, 1.0),
	// 		vec3(-1.0, 1.0, 1.0),
	// 		color,
	// 	);
	// 	self.draw_line(
	// 		projector,
	// 		vec3(-1.0, 1.0, 1.0),
	// 		vec3(-1.0, -1.0, 1.0),
	// 		color,
	// 	);
	// }

	fn draw_line(
		&mut self,
		projector: &CameraProjector,
		start: Vector3,
		end: Vector3,
		color: Color,
	) {
		// Magnify for debugging. Remove these two lines eventually.
		let start = start * 100.0;
		let end = end * 100.0;

		let (start_x, start_y, start_depth) = projector.project_point(start);
		let (start_x, start_y) = (start_x as i32, start_y as i32);
		let start_point: sdl2::rect::Point = (start_x, start_y).into();
		let (end_x, end_y, end_depth) = projector.project_point(end);
		let (end_x, end_y) = (end_x as i32, end_y as i32);
		let end_point: sdl2::rect::Point = (end_x, end_y).into();

		if (start_depth > 1.0 && end_depth > 1.0)
			|| (start_depth < 0.0 && end_depth < 0.0)
		{
			return;
		}
		// .filter(|&(_, _, depth, _)| depth > -1.0 && depth < 1.0)
		// .filter(|&(x, y, _, _)| x > 0 && y > 0)
		// .filter(|&(x, y, _, _)| x < viewport_width && y < viewport_height)

		self.canvas.set_draw_color(color);
		self.canvas.draw_line(start_point, end_point);
	}
}
