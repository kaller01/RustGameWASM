use macroquad::prelude::*;

use super::{CHUNK_SIZE, entity::Direction};

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

impl Coords {
    pub fn from_position(pos: &ChunkPosition) -> Coords {
        Coords::from_position_at(pos, (0, 0))
    }
    pub fn from_position_at(pos: &ChunkPosition, (x, y): (i32, i32)) -> Coords {
        Coords {
            x: pos.x * CHUNK_SIZE + x,
            y: pos.y * CHUNK_SIZE + y,
        }
    }
    pub fn from_vec2(vec: Vec2) -> Coords {
        Coords {
            x: vec.x.round() as i32,
            y: vec.y.round() as i32,
        }
    }
    pub fn to_vec2(&self) -> Vec2 {
        vec2(self.x as f32, self.y as f32)
    }
}

impl ChunkPosition {
    pub fn from_coords(coords: &Coords) -> ChunkPosition {
        ChunkPosition {
            x: ((coords.x as f32) / (CHUNK_SIZE as f32)).floor() as i32,
            y: ((coords.y as f32) / (CHUNK_SIZE as f32)).floor() as i32,
        }
    }
    pub fn from_rect(rect: Rect) -> (ChunkPosition, ChunkPosition) {
        (
            ChunkPosition {
                x: (rect.x / (CHUNK_SIZE as f32)).floor() as i32,
                y: (rect.y / (CHUNK_SIZE as f32)).floor() as i32,
            },
            ChunkPosition {
                x: ((rect.x + rect.w) / (CHUNK_SIZE as f32)).ceil() as i32,
                y: ((rect.y + rect.h) / (CHUNK_SIZE as f32)).ceil() as i32,
            },
        )
    }
}

pub fn get_tiles_in_half_circle(center: Coords, _direction: Direction, radius: f32) -> Vec<Coords> {
    let mut tiles = Vec::new();

    // let facing_vec = direction.to_vec2();
    // let facing_angle = facing_vec.y.atan2(facing_vec.x);
    // let half_circle_angle = std::f32::consts::PI / 2.0;

    for x in (center.x - radius as i32)..=(center.x + radius as i32) {
        for y in (center.y - radius as i32)..=(center.y + radius as i32) {
            let tile_coords = Coords { x, y };

            // Check if tile is within radius
            let distance_squared =
                (tile_coords.x - center.x).pow(2) + (tile_coords.y - center.y).pow(2);
            let radius_squared = radius.powf(2.);
            if distance_squared as f32 <= radius_squared {
                // Check if tile is within half circle facing in player direction
                // let tile_vec = Vec2 {
                //     x: (tile_coords.x - center.x) as f32,
                //     y: (tile_coords.y - center.y) as f32,
                // };
                // let tile_angle = tile_vec.y.atan2(tile_vec.x);
                // let angle_diff = (tile_angle - facing_angle).abs();
                // let within_half_circle = angle_diff <= half_circle_angle;
                // if within_half_circle {
                    tiles.push(tile_coords);
                // }
            }
        }
    }

    tiles
}