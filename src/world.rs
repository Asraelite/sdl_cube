use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::geometry::{self, vec3, Vector3};
use super::window::{Keycode, WindowInputState};

pub const FRAME_WIDTH: usize = 16;
pub const TILE_SIZE: f32 = 2.0 / FRAME_WIDTH as f32;
const FRAME_TILE_COUNT: usize = FRAME_WIDTH * FRAME_WIDTH;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Contacts {
	top_left: bool,
	top_right: bool,
	bottom_left: bool,
	bottom_right: bool,
}

impl Contacts {
	pub fn as_tuple(&self) -> (bool, bool, bool, bool) {
		(
			self.top_left,
			self.top_right,
			self.bottom_left,
			self.bottom_right,
		)
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
	Neutral,
}

pub struct World {
	frames: HashMap<FramePosition, Frame>,
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

		let start_frame = Frame::new_populated();
		let start_frame_position = FramePosition::new(0, 0);
		world.frames.insert(start_frame_position, start_frame);

		let player = Entity::new_player(&mut world, start_frame_position);
		let player_id = player.id;
		world.entities.insert(player.id, player);

		world.focus_entity = Some(player_id);

		world
	}

	pub fn tick(&mut self, input_state: &WindowInputState) {
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
				_ => {}
			}
		}

		for &keycode in input_state.keys_pressed.iter() {
			use Keycode::*;
			match keycode {
				W => {
					self.jump_entity(player_id);
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

		let mut direction_x = match step_vector.x {
			dx if dx == 0.0 => Direction::Neutral,
			dx if dx > 0.0 => Direction::Right,
			dx if dx < 0.0 => Direction::Left,
			_ => panic!("NaN velocity vector component, {:?}", step_vector),
		};

		let mut direction_y = match step_vector.y {
			dy if dy == 0.0 => Direction::Neutral,
			dy if dy > 0.0 => Direction::Down,
			dy if dy < 0.0 => Direction::Up,
			_ => panic!("NaN velocity vector component, {:?}", step_vector),
		};

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
			) {
				(Right, (true, true, _, _), (_, _, _, true), _) => true,
				(Right, (true, _, _, _), (_, _, _, true), Right) => true,
				(Right, (_, _, true, true), (_, true, _, _), _) => true,
				(Right, (_, _, true, _), (_, true, _, _), Right) => true,
				(Right, _, (_, true, _, true), _) => true,

				(Left, (true, true, _, _), (_, _, true, _), _) => true,
				(Left, (_, true, _, _), (_, _, true, _), Left) => true,
				(Left, (_, _, true, true), (true, _, _, _), _) => true,
				(Left, (_, _, _, true), (true, _, _, _), Left) => true,
				(Left, _, (true, _, true, _), _) => true,

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
				direction_x = last_direction_x;
			}

			let start_contacts = self.point_contacts(position);
			position.y += step_vector.y;
			let end_contacts = self.point_contacts(position);

			let collision_y = match (
				direction_y,
				start_contacts.as_tuple(),
				end_contacts.as_tuple(),
				last_direction_y,
			) {
				(Down, (_, true, _, true), (_, _, true, _), _) => true,
				(Down, (_, true, _, _), (_, _, true, _), Down) => true,
				(Down, (true, _, true, _), (_, _, _, true), _) => true,
				(Down, (true, _, _, _), (_, _, _, true), Down) => true,
				(Down, _, (_, _, true, true), _) => true,

				(Up, (_, true, _, true), (true, _, _, _), _) => true,
				(Up, (_, _, _, true), (true, _, _, _), Up) => true,
				(Up, (true, _, true, _), (_, true, _, _), _) => true,
				(Up, (_, _, true, _), (_, true, _, _), Up) => true,
				(Up, _, (true, true, _, _), _) => true,

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
				direction_y = last_direction_y;
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

		let entity = self.get_entity_mut(id).unwrap();
		entity.position = position;
		entity.velocity = velocity;
		entity.last_movement_direction_x = direction_x;
		entity.last_movement_direction_y = direction_y;
		entity.grounded = grounded;

		// Air friction and gravity.
		entity.velocity.x *= 0.8;
		entity.velocity.y += 0.0004;
	}

	pub fn tile_at_entity(&self, id: EntityId) -> Tile {
		let entity = self.get_entity(id).unwrap();
		self.tile_at_position(entity.position)
	}

	pub fn tile_at_position(&self, position: WorldPosition) -> Tile {
		let frame_position = position.frame;
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
		let frame = self.get_frame(position.frame).unwrap();

		let f = FRAME_WIDTH as f32 / 2.0;
		let tile_x_left = (((position.x + 1.0) * f).ceil() - 1.0) as isize;
		let tile_x_right = (((position.x + 1.0) * f).floor()) as isize;
		let tile_y_up = (((position.y + 1.0) * f).ceil() - 1.0) as isize;
		let tile_y_down = ((position.y + 1.0) * f).floor() as isize;

		let up_left_solid = frame.tile(tile_x_left, tile_y_up).is_solid();
		let up_right_solid = frame.tile(tile_x_right, tile_y_up).is_solid();
		let down_left_solid = frame.tile(tile_x_left, tile_y_down).is_solid();
		let down_right_solid = frame.tile(tile_x_right, tile_y_down).is_solid();

		Contacts {
			top_left: up_left_solid,
			top_right: up_right_solid,
			bottom_left: down_left_solid,
			bottom_right: down_right_solid,
		}

		// Collision calculation

		// let solid_tiles = (
		// 	up_left_solid,
		// 	up_right_solid,
		// 	down_left_solid,
		// 	down_right_solid,
		// );

		// If the entity or point is on the intersection of four tiles,
		// then the values of those tiles are represented in the
		// 4-tuple `solid_tiles` in the following order:
		// 0 1
		// 2 3
		// let contact_above = match solid_tiles {
		// 	(true, true, _, _) => true,
		// 	(true, _, _, true) => true,
		// 	(_, true, true, _) => true,
		// 	_ => false,
		// };

		// let contact_below = match solid_tiles {
		// 	(_, _, true, true) => true,
		// 	(_, true, true, _) => true,
		// 	(true, _, _, true) => true,
		// 	_ => false,
		// };

		// let contact_left = match solid_tiles {
		// 	(true, _, true, _) => true,
		// 	(true, _, _, true) => true,
		// 	(_, true, true, _) => true,
		// 	_ => false,
		// };

		// let contact_right = match solid_tiles {
		// 	(_, true, _, true) => true,
		// 	(_, true, true, _) => true,
		// 	(true, _, _, true) => true,
		// 	_ => false,
		// };

		// Contacts {
		// 	top: contact_above,
		// 	below: contact_below,
		// 	left: contact_left,
		// 	right: contact_right,
		// }
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

	pub fn get_frame(&self, frame_position: FramePosition) -> Option<&Frame> {
		self.frames.get(&frame_position)
	}

	pub fn get_frame_mut(
		&mut self,
		frame_position: FramePosition,
	) -> Option<&mut Frame> {
		self.frames.get_mut(&frame_position)
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FramePosition {
	pub x: usize,
	pub y: usize,
}

impl FramePosition {
	pub fn new(x: usize, y: usize) -> Self {
		Self { x, y }
	}
}

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
	Empty,
	Solid,
	Invalid,
}

impl Tile {
	pub fn is_solid(&self) -> bool {
		use Tile::*;
		match *self {
			Empty => false,
			Solid => true,
			Invalid => true,
		}
	}
}

pub struct FrameBorders {
	up: Option<FramePosition>,
	down: Option<FramePosition>,
	left: Option<FramePosition>,
	right: Option<FramePosition>,
}

impl FrameBorders {
	pub fn at_direction(&self, direction: Direction) -> Option<FramePosition> {
		use Direction::*;
		match direction {
			Up => self.up,
			Down => self.down,
			Left => self.left,
			Right => self.right,
			Neutral => None,
		}
	}
}

pub struct Frame {
	tiles: [Tile; FRAME_TILE_COUNT],
	invalid_tile: Tile,
	borders: FrameBorders,
}

impl Frame {
	pub fn new() -> Self {
		let borders = FrameBorders {
			up: None,
			down: None,
			left: None,
			right: None,
		};

		Self {
			tiles: [Tile::Empty; FRAME_TILE_COUNT],
			invalid_tile: Tile::Invalid,
			borders,
		}
	}

	pub fn tile(&self, x: isize, y: isize) -> &Tile {
		if x < 0
			|| y < 0 || x >= FRAME_WIDTH as isize
			|| y >= FRAME_WIDTH as isize
		{
			return &Tile::Invalid;
		}

		&self.tiles[y as usize * FRAME_WIDTH + x as usize]
	}

	pub fn tile_mut(&mut self, x: isize, y: isize) -> &mut Tile {
		if x < 0
			|| y < 0 || x >= FRAME_WIDTH as isize
			|| y >= FRAME_WIDTH as isize
		{
			return &mut self.invalid_tile;
		}

		&mut self.tiles[y as usize * FRAME_WIDTH + x as usize]
	}

	pub fn new_populated() -> Self {
		let mut frame = Self::new();

		let mut rng = rand::thread_rng();

		for x in 0..FRAME_WIDTH {
			for y in 0..FRAME_WIDTH {
				let tile = match rng.gen_range(1, 100) {
					1..=25 => Tile::Solid,
					26..=100 => Tile::Empty,
					_ => panic!(),
				};

				*frame.tile_mut(x as isize, y as isize) = tile;
			}
		}

		frame
	}
}

#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
	pub frame: FramePosition,
	pub x: f32,
	pub y: f32,
}

pub struct Entity {
	pub position: WorldPosition,
	pub velocity: Vector3,
	pub last_movement_direction_x: Direction,
	pub last_movement_direction_y: Direction,
	pub kind: EntityKind,
	pub orientation: Direction,
	pub id: EntityId,
	pub grounded: bool,
	//pub contacts: Contacts,
}

impl Entity {
	pub fn new_player(world: &mut World, frame: FramePosition) -> Self {
		let position = WorldPosition {
			frame,
			x: 0.3,
			y: 0.1,
		};

		let id = EntityId(world.generate_id());

		// let contacts = Contacts {
		// 	above: false,
		// 	below: false,
		// 	left: false,
		// 	right: false,
		// };

		Self {
			position,
			velocity: Vector3::zero(),
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
