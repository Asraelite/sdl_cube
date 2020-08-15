pub mod types;
mod backend;
mod projection;

use std::collections::HashSet;

use backend::Backend;

use super::world::{Direction, Entity, Frame, Tile, World, FRAME_WIDTH};
use super::GameState;
use crate::geometry::{self, vec3, Matrix4x4, Vector3, PI};

pub use types::*;

use projection::{Camera, CameraProjector};

const DEBUG_0: usize = 60;
const THREE_D_TILES: bool = false;

pub struct Window {
	backend: Backend,
	input_state: InputState,
	pub should_exit: bool,
	tick: usize,
	debug: (isize, isize),
}

pub struct InputState {
	// Keyboard keys that started being pressed this frame
	pub keys_pressed: HashSet<Keycode>,
	// Keyboard keys that have not yet been released, regardless of when
	// they started being pressed.
	pub keys_held: HashSet<Keycode>,
}

impl InputState {
	pub fn new() -> Self {
		Self {
			keys_pressed: HashSet::new(),
			keys_held: HashSet::new(),
		}
	}

	pub fn key_down_event(&mut self, keycode: Keycode) {
		// SDL triggers this event on a key long-press, so handle that case.
		if self.keys_held.contains(&keycode) == false {
			self.keys_held.insert(keycode);
			self.keys_pressed.insert(keycode);
		}
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
		Self {
			backend: Backend::new(),
			input_state: InputState::new(),
			should_exit: false,
			tick: 0,
			debug: (0, 0),
		}
	}

	pub fn tick(&mut self, game_state: &mut GameState) {
		while let Some(event) = self.backend.poll_event() {
			use WindowEvent::*;
			match event {
				Quit { .. } => self.should_exit = true,
				KeyDown(Keycode::Escape) => self.should_exit = true,
				KeyDown(keycode) => self.input_state.key_down_event(keycode),
				KeyUp(keycode) => self.input_state.key_up_event(keycode),
				_ => {}
			}
		}

		game_state.tick(&self.input_state);
		self.input_state.clear_frame();
		self.tick += 1;
	}

	pub fn render(&mut self, game_state: &mut GameState) {
		self.backend.clear_canvas();

		let projector = {
			let position = Vector3::new(0.0, 0.0, 240.0);
			let rotation = Vector3::new(0.0, 0.0, 0.0);
			let fov_degrees = 50.0;
			let camera = Camera::new(position, rotation, fov_degrees);

			let viewport_width = self.backend.viewport_width() as f32;
			let viewport_height = self.backend.viewport_width() as f32;

			camera.projector(viewport_width, viewport_height)
		};

		self.render_cube(&projector, game_state);

		self.backend.update_canvas();
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

		// let debug_tile_pos = world.tile_index_at_entity(focus_entity.id);
		// self.debug = debug_tile_pos;

		let focus_x =
			focus_position.x.abs().powf(1.5).copysign(focus_position.x);
		let focus_y =
			focus_position.y.abs().powf(1.5).copysign(focus_position.y);

		// let r = vec3(
		// 	focus_y * (PI / 4.0),
		// 	focus_x * -(PI / 4.0),
		// 	0.0,
		// );

		// let view_rotate_y = focus_x * (-PI / 4.0);
		// let view_rotate_x = vec3(focus_x, focus_y, 1.0).normalized().y;
		// let view_rotate_x = view_rotate_x * (PI / 3.0);

		// Convert Cartesian coordinates on the cube into spherical
		// coordinates.
		let view_rotate_y = focus_x.atan();
		let view_rotate_x =
			(PI / 4.0 * 2.0) - ((focus_x.powi(2) + 1.0).sqrt()).atan2(focus_y);
		// let view_rotate_x = 0.0;

		let r = vec3(view_rotate_x, -view_rotate_y, 0.0);
		//let focus_vec = vec3(focus_x, focus_y, 1.0).normalized();
		//let spin = 0.0;
		//println!("{}", );
		//let r = r.rotated(0.0, 0.0, 0.5);
		//let r =

		let focus_frame = world.get_frame(focus_position.frame).unwrap();
		let neighbors = focus_frame.borders;

		let view_rotation = Matrix4x4::rotation(r.x, r.y, r.z);

		let mut twist = 0.0;

		if focus_y > 0.0 {
			twist = (PI * 2.0) / 6.0;
		}

		//let twist = (self.tick as f32) / 300.0;
		let view_rotation = {
			let p = vec3(focus_x, focus_y, 1.0).normalized();
			view_rotation.rotated_about_axis(p, twist)
		};
		type DrawFrameFn =
			fn(&mut Window, &CameraProjector, &Frame, Direction, Matrix4x4);

		let mut frames_do = |f: DrawFrameFn| {
			for &direction in Direction::iter() {
				let neighbor = neighbors.at_direction(direction);
				if let Some(neighbor) = neighbor {
					let frame = world.get_frame(neighbor.frame).unwrap();
					f(self, projector, &frame, direction, view_rotation);
				}
			}
		};

		frames_do(Self::draw_frame_border);
		frames_do(Self::draw_frame_interior);

		// self.draw_frame(projector, &frame, Direction::Neutral, r);
		// self.draw_frame(projector, &frame, Direction::Up, r);
		// self.draw_frame(projector, &frame, Direction::Down, r);
		// self.draw_frame(projector, &frame, Direction::Left, r);
		// self.draw_frame(projector, &frame, Direction::Right, r);

		for entity_id in world.entity_ids() {
			let entity = world.get_entity(entity_id).unwrap();
			let frame = entity.position.frame;
			self.draw_entity(
				projector,
				entity,
				Direction::Neutral,
				view_rotation,
			);
		}
	}

	fn draw_entity(
		&mut self,
		projector: &CameraProjector,
		entity: &Entity,
		direction: Direction,
		view_rotation: Matrix4x4,
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

		let mut direction_rotation =
			Matrix4x4::rotation(rotate_pitch, rotate_roll, 0.0);

		let r = view_rotation * direction_rotation;
		let p = entity.position;
		self.draw_line(
			projector,
			vec3(p.x, p.y - 0.01, 1.00) * r,
			vec3(p.x, p.y + 0.01, 1.00) * r,
			Color::CYAN,
		);
	}

	fn frame_rotation_matrix(
		&self,
		projector: &CameraProjector,
		direction: Direction,
		view_rotation: Matrix4x4,
	) -> Matrix4x4 {
		let (mut rotate_pitch, mut rotate_roll) = match direction {
			Direction::Neutral => (0.0, 0.0),
			Direction::Up => (PI / 2.0, 0.0),
			Direction::Down => (-PI / 2.0, 0.0),
			Direction::Right => (0.0, PI / 2.0),
			Direction::Left => (0.0, -PI / 2.0),
			_ => (0.0, 0.0),
		};

		let mut direction_rotation =
			Matrix4x4::rotation(rotate_pitch, rotate_roll, 0.0);

		direction_rotation
	}

	fn draw_frame_border(
		&mut self,
		projector: &CameraProjector,
		frame: &Frame,
		direction: Direction,
		view_rotation: Matrix4x4,
	) {
		let color = Color::GRAY;

		let direction_rotation =
			self.frame_rotation_matrix(projector, direction, view_rotation);

		let m = direction_rotation;
		let r = view_rotation;
		let p1 = vec3(-1.0, -1.0, 1.0) * m * r;
		let p2 = vec3(1.0, -1.0, 1.0) * m * r;
		let p3 = vec3(1.0, 1.0, 1.0) * m * r;
		let p4 = vec3(-1.0, 1.0, 1.0) * m * r;

		if self.is_rect_visible(projector, p1, p2, p3, p4) == false {
			return;
		}

		self.draw_rect(projector, p1, p2, p3, p4, color);
	}

	fn draw_frame_interior(
		&mut self,
		projector: &CameraProjector,
		frame: &Frame,
		direction: Direction,
		view_rotation: Matrix4x4,
	) {
		let color = Color::WHITE;

		let direction_rotation =
			self.frame_rotation_matrix(projector, direction, view_rotation);

		let m = direction_rotation;
		let r = view_rotation;

		let p1 = vec3(-1.0, -1.0, 1.0) * m * r;
		let p2 = vec3(1.0, -1.0, 1.0) * m * r;
		let p3 = vec3(1.0, 1.0, 1.0) * m * r;
		let p4 = vec3(-1.0, 1.0, 1.0) * m * r;

		if self.is_rect_visible(projector, p1, p2, p3, p4) == false {
			return;
		}

		//self.draw_rect(projector, p1, p2, p3, p4, color);

		let f = 1.0 / FRAME_WIDTH as f32;
		for x in 0..FRAME_WIDTH {
			for y in 0..FRAME_WIDTH {
				let mut o = vec3(
					(x as f32 / FRAME_WIDTH as f32 * 2.0),
					(y as f32 / FRAME_WIDTH as f32 * 2.0),
					0.0,
				);
				o = o - vec3(1.0, 1.0, 0.0);
				//let o = Vector3::zero();

				let mut will_render = match *frame.tile(x as isize, y as isize)
				{
					Tile::Solid => true,
					_ => false,
				};

				//println!("{:?}", self.debug);
				// let color = if (x, y) == self.debug {
				// 	will_render = true;
				// 	Color::RED
				// } else {
				// 	color
				// };

				if will_render && THREE_D_TILES {
					// depth
					let d = 0.08;
					// front
					self.draw_rect(
						projector,
						(vec3(0.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						(vec3(2.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						(vec3(0.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						color,
					);
					// top
					self.draw_rect(
						projector,
						(vec3(0.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						(vec3(0.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						color,
					);
					// left
					self.draw_rect(
						projector,
						(vec3(0.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(0.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						(vec3(0.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						(vec3(0.0 * f, 2.0 * f, 1.00) + o) * m * r,
						color,
					);
					// bottom
					self.draw_rect(
						projector,
						(vec3(0.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00) + o) * m * r,
						(vec3(0.0 * f, 2.0 * f, 1.00) + o) * m * r,
						color,
					);
					// right
					self.draw_rect(
						projector,
						(vec3(2.0 * f, 0.0 * f, 1.00 + d) + o) * m * r,
						(vec3(2.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00 + d) + o) * m * r,
						color,
					);
				} else if will_render {
					self.draw_rect(
						projector,
						(vec3(0.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 0.0 * f, 1.00) + o) * m * r,
						(vec3(2.0 * f, 2.0 * f, 1.00) + o) * m * r,
						(vec3(0.0 * f, 2.0 * f, 1.00) + o) * m * r,
						color,
					);
				}
			}
		}

		self.backend.draw_line((10.0, 10.0), (12.0, 12.0));
		//self.backend.draw_line((10, 12), (12, 12));
	}

	fn draw_rect(
		&mut self,
		projector: &CameraProjector,
		top_left: Vector3,
		top_right: Vector3,
		bottom_right: Vector3,
		bottom_left: Vector3,
		color: Color,
	) {
		let p1 = top_left;
		let p2 = top_right;
		let p3 = bottom_right;
		let p4 = bottom_left;

		if self.is_rect_visible(projector, p1, p2, p3, p4) == false {
			return;
		}

		self.draw_lines(projector, &[top_left, top_right, bottom_right], color);
		self.draw_lines(
			projector,
			&[top_left, bottom_left, bottom_right],
			color,
		);
	}

	fn is_rect_visible(
		&self,
		projector: &CameraProjector,
		top_left: Vector3,
		top_right: Vector3,
		bottom_right: Vector3,
		bottom_left: Vector3,
	) -> bool {
		//return true;
		let p1 = projector.apply_projection_matrix(top_left * 100.0);
		let p2 = projector.apply_projection_matrix(top_right * 100.0);
		let p3 = projector.apply_projection_matrix(bottom_right * 100.0);
		let normal = geometry::normal(p1, p2, p3);
		normal.z >= 0.0
	}

	fn draw_lines(
		&mut self,
		projector: &CameraProjector,
		points: &[Vector3],
		color: Color,
	) {
		let projected_points: Vec<(f32, f32)> = points
			.iter()
			.map(|point| {
				// Magnify for debugging. `* 100.0` should be removed eventually.
				let (x, y, depth) = projector.project_point(*point * 100.0);
				(x, y)
			})
			.collect();

		self.backend.set_draw_color(color);
		self.backend.draw_lines(projected_points.as_slice());
		//self.backend.draw_line(end_point, start_point);
	}

	fn draw_line(
		&mut self,
		projector: &CameraProjector,
		start: Vector3,
		end: Vector3,
		color: Color,
	) {
		self.draw_lines(projector, &[start, end], color);
	}
}
