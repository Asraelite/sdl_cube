use std::collections::HashMap;

use crate::backend::random;
use crate::prelude::*;

use super::geometry::{self, vec3, Vector3};
use super::window::InputState;
use super::window::{Color, Keycode};

mod types;
pub use types::*;
mod frame;
pub use frame::{Frame, FrameLink};

pub const FRAME_WIDTH: usize = 16;
pub const TILE_SIZE: f32 = 2.0 / FRAME_WIDTH as f32;
const FRAME_TILE_COUNT: usize = FRAME_WIDTH * FRAME_WIDTH;

pub struct World {
	frames: HashMap<FrameId, Frame>,
	entities: HashMap<EntityId, Entity>,
	pub focus_entity: Option<EntityId>,
	iota: usize,
}

impl World {
	pub fn new() -> Self {
		let mut world = Self {
			frames: HashMap::new(),
			entities: HashMap::new(),
			focus_entity: None,
			iota: 0,
		};

		let front = Frame::new_populated(FrameId::new(0));
		let front_id = world.insert_frame(front);

		let left = Frame::new_populated(FrameId::new(1));
		let left_id = world.insert_frame(left);

		let right = Frame::new_populated(FrameId::new(2));
		let right_id = world.insert_frame(right);

		let up = Frame::new_populated(FrameId::new(3));
		let up_id = world.insert_frame(up);

		let down = Frame::new_populated(FrameId::new(4));
		let down_id = world.insert_frame(down);

		let back = Frame::new_populated(FrameId::new(5));
		let back_id = world.insert_frame(back);

		use Direction::*;

		world.connect_frames(front_id, Up, up_id, Down);
		world.connect_frames(front_id, Left, left_id, Right);
		world.connect_frames(front_id, Right, right_id, Left);
		world.connect_frames(front_id, Down, down_id, Up);

		world.connect_frames(back_id, Up, up_id, Up);
		world.connect_frames(back_id, Right, left_id, Left);
		world.connect_frames(back_id, Left, right_id, Right);
		world.connect_frames(back_id, Down, down_id, Down);

		world.connect_frames(left_id, Up, up_id, Left);
		world.connect_frames(left_id, Down, down_id, Right);

		world.connect_frames(right_id, Up, up_id, Right);
		world.connect_frames(right_id, Down, down_id, Left);

		let player = Entity::new_player(&mut world, front_id);
		let player_id = player.id;
		world.entities.insert(player.id, player);

		world.focus_entity = Some(player_id);

		world
	}

	fn insert_frame(&mut self, frame: Frame) -> FrameId {
		let id = frame.position;
		self.frames.insert(id, frame);
		id
	}

	pub fn tick(&mut self, input_state: &InputState) {
		let player_id = self.focus_entity.unwrap();

		let speed = 0.002;

		for &keycode in input_state.keys_held.iter() {
			use Keycode::*;
			match keycode {
				A => {
					self.impulse_entity(player_id, vec3(-speed, 0.0, 0.0));
				}
				D => {
					self.impulse_entity(player_id, vec3(speed, 0.0, 0.0));
				}
				W => {
					self.impulse_entity(player_id, vec3(0.0, -speed, 0.0));
				}
				S => {
					self.impulse_entity(player_id, vec3(0.0, speed, 0.0));
				}
				_ => {}
			}
		}

		for &keycode in input_state.keys_pressed.iter() {
			use Keycode::*;
			match keycode {
				W => {
					self.jump_entity(player_id);
				}
				E => {
					let entity = self.get_entity(player_id).unwrap();
					let mut position = entity.position;
					let (ex, ey) = self.tile_index_at_position(position);
					let (tile_frame_position, tx, ty) = self
						.normalize_tile_index(position.frame_id, ex + 1, ey);
					let frame =
						self.get_frame_mut(tile_frame_position).unwrap();
					*frame.tile_mut(tx, ty) = Tile::Solid;
				}
				Q => {
					let entity = self.get_entity(player_id).unwrap();
					let mut position = entity.position;
					let (ex, ey) = self.tile_index_at_position(position);
					let (tile_frame_position, tx, ty) = self
						.normalize_tile_index(position.frame_id, ex + 1, ey);
					let frame =
						self.get_frame_mut(tile_frame_position).unwrap();
					*frame.tile_mut(tx, ty) = Tile::Empty;
				}
				_ => {}
			}
		}
		for id in self.entity_ids() {
			self.move_entity(id);
		}
	}

	// Change current position by current velocity and resolve collisions.
	fn move_entity(&mut self, id: EntityId) {
		let entity = self.get_entity_mut(id).unwrap();

		// Move in smaller steps if the magnitude of the velocity is greater
		// than the size of one tile. This does not fully eliminate clipping
		// but should reduce it.
		let iterations = (entity.velocity.len() / TILE_SIZE).max(1.0).ceil();
		let step_vector = entity.velocity / iterations;
		let last_direction_x = entity.last_movement_direction_x;
		let last_direction_y = entity.last_movement_direction_y;
		let last_direction = entity.last_movement_direction;

		let direction_x = match step_vector.x {
			dx if dx == 0.0 => Direction::Neutral,
			dx if dx > 0.0 => Direction::Right,
			dx if dx < 0.0 => Direction::Left,
			_ => panic!("NaN velocity vector component, {:?}", step_vector),
		};
		let mut set_direction_x = direction_x;

		let direction_y = match step_vector.y {
			dy if dy == 0.0 => Direction::Neutral,
			dy if dy > 0.0 => Direction::Down,
			dy if dy < 0.0 => Direction::Up,
			_ => panic!("NaN velocity vector component, {:?}", step_vector),
		};
		let mut set_direction_y = direction_y;

		let f = FRAME_WIDTH as f32 / 2.0;
		let mut position = entity.position;
		let mut velocity = entity.velocity;
		// `entity` is dropped here, allowing more references to `self`.
		let mut grounded = false;
		for _ in 0..iterations as usize {
			use Direction::*;

			let start_contacts = self.point_contacts(position);
			position.x += step_vector.x;
			let end_contacts = self.point_contacts(position);

			let collision_x = match (
				direction_x,
				start_contacts.as_tuple(),
				end_contacts.as_tuple(),
				last_direction_x,
				last_direction_y,
				last_direction,
			) {
				(Right, (true, true, _, _), (_, _, _, true), _, _, _) => true,
				(Right, (true, _, _, _), (_, _, _, true), _, _, Right) => true,
				(Right, (true, _, _, _), (_, _, _, true), _, _, Up) => true,
				(Right, (_, _, true, true), (_, true, _, _), _, _, _) => true,
				(Right, (_, _, true, _), (_, true, _, _), _, _, Down) => true,
				(Right, (_, _, true, _), (_, true, _, _), _, _, Right) => true,
				(Right, _, (_, true, _, true), _, _, _) => true,

				(Left, (true, true, _, _), (_, _, true, _), _, _, _) => true,
				(Left, (_, true, _, _), (_, _, true, _), _, _, Left) => true,
				(Left, (_, true, _, _), (_, _, true, _), _, _, Up) => true,
				(Left, (_, _, true, true), (true, _, _, _), _, _, _) => true,
				(Left, (_, _, _, true), (true, _, _, _), _, _, Left) => true,
				(Left, (_, _, _, true), (true, _, _, _), _, _, Down) => true,
				(Left, _, (true, _, true, _), _, _, _) => true,

				_ => false,
			};

			if collision_x {
				match direction_x {
					Right => position.x = (position.x * f).floor() / f,
					Left => position.x = (position.x * f).ceil() / f,
					_ => panic!(),
				}
				velocity.x = 0.0;
			} else {
				set_direction_x = last_direction_x;
			}

			let start_contacts = self.point_contacts(position);
			position.y += step_vector.y;
			let end_contacts = self.point_contacts(position);

			let collision_y = match (
				direction_y,
				start_contacts.as_tuple(),
				end_contacts.as_tuple(),
				last_direction_y,
				last_direction_x,
				last_direction,
			) {
				(Down, (_, true, _, true), (_, _, true, _), _, _, _) => true,
				(Down, (_, true, _, _), (_, _, true, _), _, _, Down) => true,
				(Down, (_, true, _, _), (_, _, true, _), _, _, Right) => true,
				(Down, (true, _, true, _), (_, _, _, true), _, _, _) => true,
				(Down, (true, _, _, _), (_, _, _, true), _, _, Down) => true,
				(Down, (true, _, _, _), (_, _, _, true), _, _, Left) => true,
				(Down, _, (_, _, true, true), _, _, _) => true,

				(Up, (_, true, _, true), (true, _, _, _), _, _, _) => true,
				(Up, (_, _, _, true), (true, _, _, _), _, _, Up) => true,
				(Up, (_, _, _, true), (true, _, _, _), _, _, Right) => true,
				(Up, (true, _, true, _), (_, true, _, _), _, _, _) => true,
				(Up, (_, _, true, _), (_, true, _, _), _, _, Up) => true,
				(Up, (_, _, true, _), (_, true, _, _), _, _, Left) => true,
				(Up, _, (true, true, _, _), _, _, _) => true,

				_ => false,
			};

			if collision_y {
				match direction_y {
					Down => {
						position.y = (position.y * f).floor() / f;
						grounded = true;
					}
					Up => position.y = (position.y * f).ceil() / f,
					_ => panic!(),
				}
				velocity.y = 0.0;
			} else {
				set_direction_y = last_direction_y;
			}

			// 		position.x = (position.x * f).floor() / f;
			// 		velocity.x = 0.0;
			// 	}
			// 	dx if dx < 0.0 => {
			// 		position.x += dx;
			// 		match self.point_contacts(position).left {
			// 			Tile::Empty => {}
			// 			_ => {
			// 				position.x = (position.x * f).ceil() / f;
			// 				velocity.x = 0.0;
			// 			}
			// 		};
			// 	}
			// 	_ => panic!("NaN velocity vector component"),
			// }
			// Most recent:
			// match step_vector.y {
			// 	dy if dy == 0.0 => {}
			// 	dy if dy > 0.0 => {
			// 		position.y += dy;
			// 		match self.point_contacts(position).below {
			// 			Tile::Empty => {}
			// 			_ => {
			// 				position.y = (position.y * f).floor() / f;
			// 				velocity.y = 0.0;
			// 			}
			// 		};
			// 	}
			// 	dy if dy < 0.0 => {
			// 		position.y += dy;
			// 		match self.point_contacts(position).above {
			// 			Tile::Empty => {}
			// 			_ => {
			// 				position.y = (position.y * f).ceil() / f;
			// 				velocity.y = 0.0;
			// 			}
			// 		};
			// 	}
			// 	_ => panic!("NaN velocity vector component"),
			// }
			// match step_vector.y {
			// 	dy if dy == 0.0 => {}
			// 	dy if dy > 0.0 => {
			// 		position.y += dy;
			// 		match self.tile_at_position(position) {
			// 			Tile::Empty => {}
			// 			_ => {
			// 				position.y = (position.y * f).floor() / f;
			// 				velocity.y = 0.0;
			// 			}
			// 		};
			// 	}
			// 	dy if dy < 0.0 => {
			// 		position.y += dy;
			// 		match self.tile_at_position(position) {
			// 			Tile::Empty => {}
			// 			_ => {
			// 				position.y = (position.y * f).ceil() / f;
			// 				velocity.y = 0.0;
			// 			}
			// 		};
			// 	}
			// 	_ => panic!("NaN velocity vector component"),
			// }
		}

		let normalized_position = position.normalize(self);
		let entity = self.get_entity_mut(id).unwrap();
		entity.position = normalized_position;
		entity.velocity = velocity;
		entity.last_movement_direction_x = set_direction_x;
		entity.last_movement_direction_y = set_direction_y;

		//println!("{:?}", normalized_position);

		// If the entity moved along both x and y this frame, y gets
		// priority.
		entity.last_movement_direction = match (direction_x, direction_y) {
			(Direction::Neutral, Direction::Neutral) => {
				entity.last_movement_direction
			}
			(x, Direction::Neutral) => x,
			(_, y) => y,
		};

		entity.grounded = grounded;

		// Air friction and gravity.
		entity.velocity.x *= 0.8;
		entity.velocity.y *= 0.8;

		if entity.velocity.x.abs() < 0.00001 {
			entity.velocity.x = 0.0;
		}
		if entity.velocity.y.abs() < 0.00001 {
			entity.velocity.y = 0.0;
		}
		//entity.velocity.y += 0.0004;
	}

	pub fn tile_at_entity(&self, id: EntityId) -> Tile {
		let entity = self.get_entity(id).unwrap();
		self.tile_at_position(entity.position)
	}

	pub fn tile_at_position(&self, position: WorldPosition) -> Tile {
		let frame_position = position.frame_id;
		let frame = self.get_frame(frame_position).unwrap();
		let (tx, ty) = self.tile_index_at_position(position);
		*frame.tile(tx, ty)
	}

	pub fn tile_index_at_entity(&self, id: EntityId) -> (isize, isize) {
		let entity = self.get_entity(id).unwrap();
		self.tile_index_at_position(entity.position)
	}

	pub fn tile_index_at_position(
		&self,
		position: WorldPosition,
	) -> (isize, isize) {
		let tx =
			((position.x + 1.0) * FRAME_WIDTH as f32 / 2.0).floor() as isize;
		let ty =
			((position.y + 1.0) * FRAME_WIDTH as f32 / 2.0).floor() as isize;

		(tx, ty)
	}

	pub fn normalize_tile_index(
		&self,
		origin_frame_position: FrameId,
		x: isize,
		y: isize,
	) -> (FrameId, isize, isize) {
		let origin_frame = self.get_frame(origin_frame_position).unwrap();
		let borders = origin_frame.borders;

		let w = FRAME_WIDTH as isize;

		if (x >= w || x < 0) && (y >= w || y < 0)
			|| (x >= w * 2 || x < -w * 2 || y >= w * 2 || y < -w * 2)
		{
			panic!(
				"Tile index exists outside its own frame \
				and orthgonally neighboring frames"
			);
		}

		use Direction::*;
		let (direction, real_x, real_y) = match (x, y) {
			(x, y) if (x >= w) => (Right, x - w, y),
			(x, y) if (x < 0) => (Left, x + w, y),
			(x, y) if (y >= w) => (Down, x, y - w),
			(x, y) if (y < 0) => (Up, x, y + w),
			(x, y) => (Neutral, x, y),
		};

		let real_frame_position = match borders.at_direction(direction) {
			Some(p) => p,
			None => {
				elog("Could not access tile index's real frame:");
				elog(format!(
					"{}/({},{}) -> {:?}",
					origin_frame_position, x, y, direction
				));
				elog(format!("selecting from {}", borders));
				panic!("Tile index access error");
			}
		}
		.frame;

		(real_frame_position, real_x, real_y)
	}

	fn jump_entity(&mut self, id: EntityId) -> bool {
		let jump_speed = 0.018;

		if self.entity_grounded(id) {
			self.get_entity_mut(id).unwrap().velocity.y = -jump_speed;
			true
		} else {
			false
		}
	}

	fn impulse_entity(&mut self, id: EntityId, vector: Vector3) {
		self.get_entity_mut(id).unwrap().velocity += vector;
	}

	fn point_contacts(&mut self, position: WorldPosition) -> Contacts {
		let frame = self.get_frame(position.frame_id).unwrap();
		let position = position.normalize(self);

		let f = FRAME_WIDTH as f32 / 2.0;
		let tile_x_left = (((position.x + 1.0) * f).ceil() - 1.0) as isize;
		let tile_x_right = (((position.x + 1.0) * f).floor()) as isize;
		let tile_y_up = (((position.y + 1.0) * f).ceil() - 1.0) as isize;
		let tile_y_down = ((position.y + 1.0) * f).floor() as isize;

		let is_solid = |x, y| {
			let (tile_frame_pos, wrapped_x, wrapped_y) =
				self.normalize_tile_index(position.frame_id, x, y);
			let tile_frame = self.get_frame(tile_frame_pos).unwrap();
			let tile = tile_frame.tile(wrapped_x, wrapped_y);
			tile.is_solid()
		};

		let up_left_solid = is_solid(tile_x_left, tile_y_up);
		let up_right_solid = is_solid(tile_x_right, tile_y_up);
		let down_left_solid = is_solid(tile_x_left, tile_y_down);
		let down_right_solid = is_solid(tile_x_right, tile_y_down);

		// let up_left_solid = frame.tile(tile_x_left, tile_y_up).is_solid();
		// let up_right_solid = frame.tile(tile_x_right, tile_y_up).is_solid();
		// let down_left_solid = frame.tile(tile_x_left, tile_y_down).is_solid();
		// let down_right_solid = frame.tile(tile_x_right, tile_y_down).is_solid();

		Contacts {
			top_left: up_left_solid,
			top_right: up_right_solid,
			bottom_left: down_left_solid,
			bottom_right: down_right_solid,
		}
	}

	fn entity_grounded(&mut self, id: EntityId) -> bool {
		let entity = self.get_entity(id).unwrap();
		entity.grounded
	}

	pub fn generate_id(&mut self) -> usize {
		let current = self.iota;
		self.iota += 1;
		current
	}

	pub fn get_entity(&self, entity_id: EntityId) -> Option<&Entity> {
		self.entities.get(&entity_id)
	}

	pub fn entity_ids(&self) -> Vec<EntityId> {
		self.entities.iter().map(|(_, ent)| ent.id).collect()
	}

	pub fn get_entity_mut(
		&mut self,
		entity_id: EntityId,
	) -> Option<&mut Entity> {
		self.entities.get_mut(&entity_id)
	}

	pub fn get_frame(&self, frame_position: FrameId) -> Option<&Frame> {
		self.frames.get(&frame_position)
	}

	pub fn get_frame_mut(
		&mut self,
		frame_position: FrameId,
	) -> Option<&mut Frame> {
		self.frames.get_mut(&frame_position)
	}

	fn connect_frames(
		&mut self,
		parent: FrameId,
		parent_edge: Direction,
		child: FrameId,
		child_edge: Direction,
	) {
		let parent_frame = self.get_frame_mut(parent).unwrap();
		let border = parent_frame.borders.at_direction_mut(parent_edge);
		if border.is_some() {
			elog(format!(
				"Attempt to create link to non-empty parent frame border:\n\
				<{}>@{:?} <- {}@{:?}\n\
				parent has: {}",
				parent, parent_edge, child, child_edge, parent_frame.borders,
			));
			panic!("Non-empty frame border attachment");
		}
		*border = Some(FrameLink {
			frame: child,
			entry_edge: child_edge,
		});
		let child_frame = self.get_frame_mut(child).unwrap();
		let border = child_frame.borders.at_direction_mut(child_edge);
		if border.is_some() {
			elog(format!(
				"Attempt to create link to non-empty child frame border:\n\
				<{}>@{:?} <- {}@{:?}\n\
				child has: {}",
				parent, parent_edge, child, child_edge, child_frame.borders,
			));
			panic!("Non-empty frame border attachment");
		}
		*border = Some(FrameLink {
			frame: parent,
			entry_edge: parent_edge,
		});
	}
}

pub struct Entity {
	pub position: WorldPosition,
	pub velocity: Vector3,
	pub last_movement_direction: Direction,
	pub last_movement_direction_x: Direction,
	pub last_movement_direction_y: Direction,
	pub kind: EntityKind,
	pub orientation: Direction,
	pub id: EntityId,
	pub grounded: bool,
	//pub contacts: Contacts,
}

impl Entity {
	pub fn new_player(world: &mut World, frame_id: FrameId) -> Self {
		let position = WorldPosition {
			frame_id,
			x: 0.3,
			y: 0.1,
		};

		let id = EntityId(world.generate_id());

		Self {
			position,
			velocity: Vector3::zero(),
			last_movement_direction: Direction::Neutral,
			last_movement_direction_x: Direction::Neutral,
			last_movement_direction_y: Direction::Neutral,
			kind: EntityKind::Player,
			orientation: Direction::Up,
			id,
			grounded: false,
			//contacts,
		}
	}
}

pub enum EntityKind {
	Player,
}
