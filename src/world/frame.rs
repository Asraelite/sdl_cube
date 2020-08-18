use crate::backend::random;
use super::{FRAME_WIDTH, TILE_SIZE, FRAME_TILE_COUNT};
use super::types::*;

#[derive(Copy, Clone, Debug)]
pub struct FrameLinks {
	pub up: Option<FrameLink>,
	pub down: Option<FrameLink>,
	pub left: Option<FrameLink>,
	pub right: Option<FrameLink>,
	pub neutral: Option<FrameLink>,
}

impl std::fmt::Display for FrameLinks {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let inner_string = [
			(/*"←"*/ Direction::Left, self.left),
			(/*"→"*/ Direction::Right, self.right),
			(/*"↑"*/ Direction::Up, self.up),
			(/*"↓"*/ Direction::Down, self.down),
		]
		.iter()
		.filter(|(_, value)| value.is_some())
		.map(|(symbol, value)| format!("{:?}:{}", symbol, value.unwrap().frame))
		.collect::<Vec<String>>()
		.join(", ");

		write!(f, "links( {} )", inner_string)?;
		Ok(())
	}
}

#[derive(Copy, Clone, Debug)]
pub struct FrameLink {
	pub frame: FrameId,
	pub entry_edge: Direction,
}

impl std::fmt::Display for FrameLink {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "~({}@{:?})", self.frame, self.entry_edge)?;
		Ok(())
	}
}

impl FrameLinks {
	pub fn at_direction(&self, direction: Direction) -> Option<FrameLink> {
		use Direction::*;
		match direction {
			Up => self.up,
			Down => self.down,
			Left => self.left,
			Right => self.right,
			Neutral => self.neutral,
		}
	}

	pub fn at_direction_mut(
		&mut self,
		direction: Direction,
	) -> &mut Option<FrameLink> {
		use Direction::*;
		match direction {
			Up => &mut self.up,
			Down => &mut self.down,
			Left => &mut self.left,
			Right => &mut self.right,
			Neutral => &mut self.neutral,
		}
	}
}


pub struct Frame {
	tiles: [Tile; FRAME_TILE_COUNT],
	invalid_tile: Tile,
	pub borders: FrameLinks,
	pub position: FrameId,
	pub orientation: Direction,
}

impl Frame {
	pub fn new(position: FrameId) -> Self {
		let borders = FrameLinks {
			up: None,
			down: None,
			left: None,
			right: None,
			neutral: Some(FrameLink {
				frame: position,
				entry_edge: Direction::Neutral,
			}),
		};

		Self {
			tiles: [Tile::Empty; FRAME_TILE_COUNT],
			invalid_tile: Tile::Invalid,
			borders,
			position,
			orientation: Direction::Neutral,
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

	pub fn new_populated(position: FrameId) -> Self {
		let mut frame = Self::new(position);

		for x in 0..FRAME_WIDTH {
			for y in 0..FRAME_WIDTH {
				let tile = match random::rangei(1, 100) {
					1..=17 => Tile::Solid,
					18..=100 => Tile::Empty,
					_ => panic!(),
				};

				*frame.tile_mut(x as isize, y as isize) = tile;
			}
		}

		frame
	}
}
