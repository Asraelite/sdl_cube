use std::collections::HashMap;

// rng.gen_range(-5000..=5000) as f64;

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

		let start_frame = Frame::new();
		let start_frame_position = FramePosition::new(0, 0);
		world.frames.insert(start_frame_position, start_frame);

		let player = Entity::new_player(&mut world, start_frame_position);
		let player_id = player.id;
		world.entities.insert(player.id, player);

		world.focus_entity = Some(player_id);

		world
	}

	pub fn generate_id(&mut self) -> usize {
		let current = self.iota;
		self.iota += 1;
		current
	}

	pub fn get_entity(&self, entity_id: EntityId) -> Option<&Entity> {
		self.entities.get(&entity_id)
	}

	pub fn get_frame(&self, frame_position: FramePosition) -> Option<&Frame> {
		self.frames.get(&frame_position)
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

pub const FRAME_WIDTH: usize = 128;
const FRAME_TILE_COUNT: usize = FRAME_WIDTH * FRAME_WIDTH;

#[derive(Copy, Clone, PartialEq)]
pub enum Tile {
	Empty,
	Solid,
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
			borders,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
	pub frame: FramePosition,
	pub x: f32,
	pub y: f32,
}

pub enum Direction {
	Up,
	Down,
	Left,
	Right,
	Neutral,
}

pub struct Entity {
	pub position: WorldPosition,
	pub kind: EntityKind,
	pub orientation: Direction,
	pub id: EntityId,
}

impl Entity {
	pub fn new_player(world: &mut World, frame: FramePosition) -> Self {
		let position = WorldPosition {
			frame,
			x: 0.3,
			y: 0.1,
		};

		let id = EntityId(world.generate_id());

		Self {
			position,
			kind: EntityKind::Player,
			orientation: Direction::Up,
			id,
		}
	}
}

pub enum EntityKind {
	Player,
}
