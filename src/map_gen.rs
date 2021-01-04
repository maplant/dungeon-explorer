//! Tile-based dungeon map generation.
use crate::kd_tree::KDTree;
use crate::rect::Rect;
use cgmath::*;
use lazy_static::lazy_static;
use rand::prelude::*;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::video::Window;

/// Type of the tile.
// TODO(map): Add more in the future, the possibilities are endless!
#[derive(Copy, Clone, Debug)]
pub enum Tile {
    Empty,
    Dirt,
}

/// Alias for empty tile.
const E: Tile = Tile::Empty;
/// Alias for dirt tile.
const D: Tile = Tile::Dirt;

impl Tile {
    fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

/// A cardinal direction.
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    fn flip(self) -> Direction {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

/// Rectangular room, with a set of entrances in each of the cardinal
/// directions.
#[derive(Clone, Debug)]
pub struct Room {
    width: u32,
    height: u32,
    /// Goes from top to bottom, west to east.
    layout: &'static [&'static [Tile]],
    /// List of open edges per each cardinal direction.
    entrances: [Vec<i32>; 4],
}

impl Room {
    fn new(layout: &'static [&'static [Tile]]) -> Self {
        let width = layout[0].len();
        let height = layout.len();
        // Find all of the entrances.
        let north_entrances: Vec<_> = layout[0][..]
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_empty())
            .map(|(i, _)| i as i32)
            .collect();
        let east_entrances: Vec<_> = layout[..]
            .iter()
            .enumerate()
            .filter(|(_, t)| t[width - 1].is_empty())
            .map(|(i, _)| i as i32)
            .collect();
        let south_entrances: Vec<_> = layout[height - 1][..]
            .iter()
            .enumerate()
            .filter(|(_, t)| t.is_empty())
            .map(|(i, _)| i as i32)
            .collect();
        let west_entrances: Vec<_> = layout[..]
            .iter()
            .enumerate()
            .filter(|(_, t)| t[0].is_empty())
            .map(|(i, _)| i as i32)
            .collect();
        // Create a texture for this.
        Self {
            width: width as u32,
            height: height as u32,
            layout,
            entrances: [
                north_entrances,
                east_entrances,
                south_entrances,
                west_entrances,
            ],
        }
    }

    fn place(&self, pos: Point2<i32>) -> RoomPlacement {
        RoomPlacement {
            pos,
            // It would be nice to avoid this clone with an Rc, but that doesn't
            // play well with lazy_static, so it would need to be an Arc, which
            // seems excessive here.
            room: self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RoomPlacement {
    pub pos: Point2<i32>,
    pub room: Room,
}

impl RoomPlacement {
    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        empty_color: (u8, u8, u8),
        dirt_color: (u8, u8, u8),
    ) {
        let mut text: Vec<_> = self
            .room
            .layout
            .iter()
            .map(|t| {
                t.iter().map(|t| {
                    if t.is_empty() {
                        [empty_color.0, empty_color.1, empty_color.2]
                    } else {
                        [dirt_color.0, dirt_color.1, dirt_color.2]
                    }
                })
            })
            .flatten()
            .flat_map(|b| b.to_vec())
            .collect();
        let surface = Surface::from_data(
            &mut text[..],
            self.room.width as u32,
            self.room.height as u32,
            self.room.width as u32 * 3,
            PixelFormatEnum::RGB24,
        )
        .unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = surface.as_texture(&texture_creator).unwrap();
        canvas
            .copy(
                &texture,
                None,
                Some(rect::Rect::new(
                    self.pos.x as i32,
                    self.pos.y as i32,
                    self.room.width as u32,
                    self.room.height as u32,
                )),
            )
            .unwrap();
    }
}

/// A tile-based dungeon map generator.
///
/// MapGenerator functions by performing a randomized depth-first search of the possibility
/// space of all possible tile-based maps, given a few stipulations regarding the connectivity
/// of rectangular spaces of tiles called "rooms".
///
#[derive(Debug)]
pub struct MapGenerator<R: Rng> {
    width: u32,
    height: u32,
    room_stack: Vec<RoomPlacement>,
    prev_placed: KDTree,
    rng: R,
}

impl<R: Rng> MapGenerator<R> {
    /// Creates a new map generator.
    pub fn new(width: u32, height: u32, rng: R) -> Self {
        let first_room = Room::new(&[
            &[E, E, E, E, E, E, E, E, E, E],
            &[E, E, E, E, E, E, E, E, E, E],
            &[E, D, D, D, E, D, D, D, D, E],
            &[E, D, D, E, E, E, E, D, D, E],
            &[E, D, E, E, E, E, E, D, E, E],
            &[E, D, E, E, E, E, E, D, E, E],
            &[E, D, E, E, E, E, E, D, E, E],
            &[E, D, D, E, E, E, E, D, E, E],
            &[E, D, D, D, E, E, E, D, D, E],
            &[D, D, D, D, E, E, D, D, D, D],
        ]);
        let start_x = ((width - first_room.width) / 2) as i32;
        let start_y = ((height - first_room.height) / 2) as i32;
        let mut kd_tree = KDTree::default();
        kd_tree.add_rect(Rect {
            min: Point2::new(start_x, start_y),
            max: Point2::new(start_x, start_y)
                + Vector2::new(first_room.width as i32, first_room.height as i32),
        });
        Self {
            width,
            height,
            room_stack: vec![first_room.place(Point2::new(start_x, start_y))],
            prev_placed: kd_tree,
            rng,
        }
    }

    /// Picks a room at random and places it, avoiding overlapping with any previously
    /// placed rooms. Returns None if no room can be placed.
    fn next_placements(&mut self, curr: &RoomPlacement) {
        // Try to attach a room to each of the entrances.
        // Create a random order of all of the available rooms and try them one-by-one
        // until one of them fits.
        let screen = Rect {
            min: Point2::new(0_i32, 0),
            max: Point2::new(self.width as i32 + 20, self.height as i32 + 20),
        };
        let mut indices: Vec<usize> = (0..AVAILABLE_ROOMS.len()).collect();
        let mut cardinals = [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ];
        cardinals.shuffle(&mut self.rng);
        for cardinal in &cardinals {
            let mut exits = curr.room.entrances[*cardinal as usize].clone();
            exits.shuffle(&mut self.rng);
            'next_exit: for exit in exits {
                indices.shuffle(&mut self.rng);
                for i in &indices {
                    // We have an exit, try the room.
                    let try_room = &AVAILABLE_ROOMS[*i];
                    for entrance in &try_room.entrances[cardinal.flip() as usize] {
                        let attempt_pos = match cardinal {
                            Direction::North => {
                                Point2::new(curr.pos.x + exit, curr.pos.y)
                                    + Vector2::new(-entrance, -(try_room.height as i32))
                            }
                            Direction::East => {
                                Point2::new(curr.pos.x + curr.room.width as i32, curr.pos.y + exit)
                                    + Vector2::new(0, -entrance)
                            }
                            Direction::South => {
                                Point2::new(curr.pos.x + exit, curr.pos.y + curr.room.height as i32)
                                    + Vector2::new(-entrance, 0)
                            }
                            Direction::West => {
                                Point2::new(curr.pos.x, curr.pos.y + exit)
                                    + Vector2::new(-(try_room.width as i32), -entrance)
                            }
                        };
                        let r = Rect {
                            min: attempt_pos,
                            max: attempt_pos
                                + Vector2::new(try_room.width as i32, try_room.height as i32),
                        };
                        if !self.prev_placed.overlaps(&r) && screen.overlaps(&r) {
                            // Push the room to the stack and add it to the kd-tree.
                            self.prev_placed.add_rect(r);
                            self.room_stack.push(try_room.place(attempt_pos));
                            continue 'next_exit;
                        }
                    }
                }
            }
        }
    }
}

/// MapGenerator is an iterator of RoomPlacements.
impl<R: Rng> std::iter::Iterator for MapGenerator<R> {
    type Item = RoomPlacement;

    fn next(&mut self) -> Option<RoomPlacement> {
        let curr_room = match self.room_stack.pop() {
            Some(room) => room,
            // No rooms left, the search has terminated.
            None => return None,
        };
        self.next_placements(&curr_room);
        Some(curr_room)
    }
}

lazy_static! {
    static ref AVAILABLE_ROOMS: Vec<Room> = vec![
        Room::new(&[&[D, D, D], &[E, E, E], &[D, D, D],]),
        Room::new(&[&[E, D, D], &[E, E, E], &[D, D, E],]),
        Room::new(&[&[D, D, E], &[E, E, E], &[E, D, D],]),
        Room::new(&[&[D, E, D], &[D, E, D], &[D, E, D],]),
        Room::new(&[&[E, E, D], &[D, E, D], &[D, E, E],]),
        Room::new(&[
            &[E, E, E, E],
            &[D, E, E, E],
            &[D, E, E, D],
            &[D, E, D, D],
            &[D, E, D, D],
            &[D, E, D, D],
            &[D, D, D, D],
        ]),
        Room::new(&[
            &[E, E, E, E, E, D, D, D],
            &[D, D, D, E, E, D, D, D],
            &[D, E, E, E, E, E, E, D],
            &[D, D, D, E, E, D, D, D],
        ]),
        Room::new(&[
            &[E, D, E, D, E, D, E, D, E],
            &[E, D, D, D, D, D, D, D, E],
            &[E, D, E, D, E, D, E, D, E],
        ]),
        Room::new(&[
            &[E, E, E, E, E, E, E, E],
            &[D, D, D, D, D, D, D, D],
            &[E, D, D, D, D, D, D, E],
            &[E, D, D, E, E, D, D, E],
        ]),
        Room::new(&[
            &[E, E, E, E, E, E, E, E, E],
            &[E, D, D, D, D, D, D, D, E],
            &[E, D, E, D, E, D, E, D, E],
            &[E, D, D, D, D, D, D, D, E],
            &[E, D, E, D, E, D, E, D, E],
            &[E, D, D, D, D, D, D, D, E],
        ]),
        Room::new(&[
            &[D, D, E, E, E, E, E, E, E],
            &[D, E, E, E, E, D, E, E, E],
            &[E, E, E, E, D, D, D, E, E],
            &[E, E, E, E, E, D, D, D, E],
            &[E, E, E, E, E, D, D, D, D],
            &[E, E, E, D, D, D, D, D, D],
        ]),
    ];
}
