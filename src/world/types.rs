use crate::prelude::*;

use super::World;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameId(pub usize);

impl std::fmt::Display for FrameId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{}]", self.0)?;
		Ok(())
	}
}

impl FrameId {
	pub fn new(inner: usize) -> Self {
		Self(inner)
	}

	pub fn invalid() -> Self {
		Self(std::usize::MAX)
	}
}

#[derive(Copy, Clone, Debug)]
pub struct WorldPosition {
	pub frame_id: FrameId,
	pub x: f32,
	pub y: f32,
}

impl WorldPosition {
	pub fn normalize(&self, world: &World) -> Self {
		RawWorldPosition {
			root_frame_id: self.frame_id,
			x: self.x,
			y: self.y,
		}
		.normalize(world)
	}
}

#[derive(Copy, Clone, Debug)]
pub struct RawWorldPosition {
	pub root_frame_id: FrameId,
	pub x: f32,
	pub y: f32,
}

impl RawWorldPosition {
	pub fn normalize(&self, world: &World) -> WorldPosition {
		let (x, y) = (self.x, self.y);
		let root_frame = world
			.get_frame(self.root_frame_id)
			.expect("Frame neighbor access error");

		if x.is_nan() || y.is_nan() {
			panic!("NaN RawWorldPosition");
		}

		if x >= -1.0 && x < 1.0 && y >= -1.0 && y < 1.0 {
			return WorldPosition {
				frame_id: self.root_frame_id,
				x,
				y,
			};
		}

		let borders = root_frame.borders;

		use Direction::*;
		let (exit_edge, real_x, real_y) = match (x, y) {
			(x, y) if (x >= 1.0) => (Right, x - 2.0, y),
			(x, y) if (x < -1.0) => (Left, x + 2.0, y),
			(x, y) if (y >= 1.0) => (Down, x, y - 2.0),
			(x, y) if (y < -1.0) => (Up, x, y + 2.0),
			(x, y) => (Neutral, x, y),
		};

		let neighbor = match borders.at_direction(exit_edge) {
			Some(p) => p,
			None => {
				elog("Could not access position's real frame:");
				elog(format!("{:?} @ {:?}", self, exit_edge));
				elog(format!("selecting from {:?}", borders));
				panic!("Frame neighbor access error");
			}
		};

		let entry_edge = neighbor.entry_edge;
		let entry_frame_id = neighbor.frame;
		let angle_change = exit_edge.angle_to(entry_edge.reverse());

		// other.rotated(self.as_angle().reverse()).as_angle()
		//println!("{:?}, {:?}", exit_edge, entry_edge.rotated(Angle::Clockwise180));
		//println!("! {:?}->{:?} '{:?}", exit_edge, entry_edge, angle_change);

		let real_world_position = RawWorldPosition {
			root_frame_id: entry_frame_id,
			x: real_x,
			y: real_y,
		}
		.rotated(angle_change);

		// Call recursively until position is brought within bounds.
		real_world_position.normalize(world)
	}

	pub fn rotated(&self, angle: Angle) -> Self {
		let (rotated_x, rotated_y) = match angle {
			Angle::Clockwise0 => (self.x, self.y),
			Angle::Clockwise90 => (-self.y, self.x),
			Angle::Clockwise180 => (-self.x, -self.y),
			Angle::Clockwise270 => (self.y, -self.x),
		};
		Self {
			root_frame_id: self.root_frame_id,
			x: rotated_x,
			y: rotated_y,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Contacts {
	pub top_left: bool,
	pub top_right: bool,
	pub bottom_left: bool,
	pub bottom_right: bool,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Angle {
	Clockwise0,
	Clockwise90,
	Clockwise180,
	Clockwise270,
}

impl Angle {
	pub fn reverse(&self) -> Self {
		use Angle::*;
		match *self {
			Clockwise0 => Clockwise180,
			Clockwise90 => Clockwise270,
			Clockwise180 => Clockwise0,
			Clockwise270 => Clockwise90,
		}
	}

	pub fn negative(&self) -> Self {
		use Angle::*;
		match *self {
			Clockwise0 => Clockwise0,
			Clockwise90 => Clockwise270,
			Clockwise180 => Clockwise180,
			Clockwise270 => Clockwise90,
		}
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

impl Direction {
	pub fn reverse(&self) -> Self {
		use Direction::*;
		match self {
			Up => Down,
			Down => Up,
			Left => Right,
			Right => Left,
			Neutral => Neutral,
		}
	}

	fn as_angle(&self) -> Angle {
		use Angle::*;
		use Direction::*;
		match self {
			Up => Clockwise0,
			Right => Clockwise90,
			Down => Clockwise180,
			Left => Clockwise270,
			Neutral => Clockwise0,
		}
	}

	pub fn angle_to(&self, other: Direction) -> Angle {
		other.rotated(self.as_angle().negative()).as_angle()
	}

	pub fn iter<'a>() -> impl std::iter::Iterator<Item = &'a Self> {
		use Direction::*;
		[Up, Down, Left, Right, Neutral].iter()
	}

	pub fn rotated(&self, angle: Angle) -> Self {
		use Angle::*;
		use Direction::*;
		match (*self, angle) {
			(Neutral, _) => Neutral,
			(current, Clockwise0) => current,
			(Up, Clockwise90) => Right,
			(Up, Clockwise180) => Down,
			(Up, Clockwise270) => Left,
			(Right, Clockwise90) => Down,
			(Right, Clockwise180) => Left,
			(Right, Clockwise270) => Up,
			(Down, Clockwise90) => Left,
			(Down, Clockwise180) => Up,
			(Down, Clockwise270) => Right,
			(Left, Clockwise90) => Up,
			(Left, Clockwise180) => Right,
			(Left, Clockwise270) => Down,
		}
	}
}
