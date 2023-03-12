use crate::player::Entity;
use macroquad::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use std::collections::HashMap;

const CHUNK_SIZE: i32 = 8;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Coords {
    x: i32,
    y: i32,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct ChunkPosition {
    x: i32,
    y: i32,
}

impl Coords {
    fn from_position(pos: &ChunkPosition) -> Coords {
        Coords::from_position_at(pos, (0, 0))
    }
    fn from_position_at(pos: &ChunkPosition, (x, y): (i32, i32)) -> Coords {
        Coords {
            x: pos.x * CHUNK_SIZE + x,
            y: pos.y * CHUNK_SIZE + y,
        }
    }
    fn from_vec2(vec: Vec2) -> Coords {
        Coords {
            x: vec.x.round() as i32,
            y: vec.y.round() as i32,
        }
    }
    fn to_vec2(&self) -> Vec2 {
        vec2(self.x as f32, self.y as f32)
    }
}

impl ChunkPosition {
    fn from_coords(coords: &Coords) -> ChunkPosition {
        ChunkPosition {
            x: ((coords.x as f32) / (CHUNK_SIZE as f32)).floor() as i32,
            y: ((coords.y as f32) / (CHUNK_SIZE as f32)).floor() as i32,
        }
    }
    fn from_rect(rect: Rect) -> (ChunkPosition, ChunkPosition) {
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

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileInteraction {
    Block,
    Walkable,
    Swimmable,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileTexture {
    Grass,
    Water,
    Sand,
    DeepWater,
    Mountain,
    SnowyMountain,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Tile {
    texture: TileTexture,
    interaction: TileInteraction,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            texture: TileTexture::Grass,
            interaction: TileInteraction::Walkable,
        }
    }
}

impl Tile {
    fn generate(n: f64) -> Tile {
        let (texture, interaction) = if n < 0.3 {
            (TileTexture::DeepWater, TileInteraction::Block)
        } else if n < 0.4 {
            (TileTexture::Water, TileInteraction::Swimmable)
        } else if n < 0.42 {
            (TileTexture::Sand, TileInteraction::Walkable)
        } else if n < 0.65 {
            (TileTexture::Grass, TileInteraction::Walkable)
        } else if n < 0.8 {
            (TileTexture::Mountain, TileInteraction::Block)
        } else {
            (TileTexture::SnowyMountain, TileInteraction::Block)
        };
        Tile {
            texture,
            interaction,
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    pos: ChunkPosition,
}

pub struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
    noise: Fbm<OpenSimplex>,
}

impl World {
    pub fn generate() -> World {
        let noise = Fbm::<OpenSimplex>::new(0)
            .set_frequency(0.01)
            .set_persistence(0.6)
            .set_lacunarity(2.0)
            .set_octaves(5);

        let x1 = -10;
        let y1 = -10;
        let x2 = 10;
        let y2 = 10;

        let mut chunks: HashMap<ChunkPosition, Chunk> = HashMap::new();

        for x in x1..x2 {
            for y in y1..y2 {
                let pos = ChunkPosition { x, y };
                let chunk = Chunk::generate(pos, &noise);
                // let chunk = Chunk::default(pos);
                chunks.insert(pos, chunk);
            }
        }
        return World { chunks, noise };
    }

    pub fn render(&self, rect: Rect) {
        let area = ChunkPosition::from_rect(rect);

        for x in area.0.x..area.1.x {
            for y in area.0.y..area.1.y {
                let pos = ChunkPosition { x, y };
                match self.chunks.get(&pos) {
                    Some(chunk) => chunk.render(),
                    None => Chunk::render_dead(pos),
                }
            }
        }
    }
    pub fn generate_at(&mut self, rect: Rect) {
        let area = ChunkPosition::from_rect(rect);
        for x in area.0.x..area.1.x {
            for y in area.0.y..area.1.y {
                let pos = ChunkPosition { x, y };
                if !self.chunks.contains_key(&pos) {
                    let chunk = Chunk::generate(pos, &self.noise);
                    self.chunks.insert(pos, chunk);
                }
            }
        }
    }
    // pub fn render_map(&self) {
    //     let scale = 1;
    //     for x in -100..100 {
    //         for y in -100..100 {
    //             let n = (self
    //                 .noise
    //                 .get_noise(((x * scale) as f32) / 300., ((y * scale) as f32) / 300.)
    //                 + 1.)
    //                 * 0.5;
    //             let tile = Tile::generate(n);
    //             let color = match tile.texture {
    //                 TileTexture::Grass => GREEN,
    //                 TileTexture::Water => BLUE,
    //                 TileTexture::DeepWater => DARKBLUE,
    //                 TileTexture::Mountain => GRAY,
    //                 TileTexture::SnowyMountain => WHITE,
    //                 TileTexture::Sand => YELLOW,
    //             };
    //             draw_rectangle(x as f32, y as f32, 1., 1., color);
    //         }
    //     }
    // }
    // fn get_tile_mut(&mut self, coords: &Coords) -> Option<&mut Tile> {
    //     let chunk_pos = ChunkPosition::from_coords(coords);
    //     let index = (
    //         (coords.x.rem_euclid(CHUNK_SIZE) as usize),
    //         (coords.y.rem_euclid(CHUNK_SIZE) as usize),
    //     );
    //     match self.chunks.get_mut(&chunk_pos) {
    //         Some(chunk) => Some(&mut chunk.tiles[index.0][index.1]),
    //         None => None,
    //     }
    // }
    fn get_tile(&self, coords: &Coords) -> Option<&Tile> {
        let chunk_pos = ChunkPosition::from_coords(coords);
        let index = (
            (coords.x.rem_euclid(CHUNK_SIZE) as usize),
            (coords.y.rem_euclid(CHUNK_SIZE) as usize),
        );
        match self.chunks.get(&chunk_pos) {
            Some(chunk) => Some(&chunk.tiles[index.0][index.1]),
            None => None,
        }
    }
    pub fn update_entity(&self, entity: &mut dyn Entity, time: f32) {
        let current_coords = Coords::from_vec2(entity.get_position());
        let mut velocity = entity.get_velocity();

        let tile = self.get_tile(&current_coords).unwrap();

        match tile.interaction {
            TileInteraction::Block => (),
            TileInteraction::Walkable => (),
            TileInteraction::Swimmable => velocity *= 0.2,
        }

        let new_pos = entity.get_position() + velocity * time;

        if self.can_move_entity_to_tile(new_pos) {
            entity.set_position(new_pos);
        } else {
            let new_pos = entity.get_position() + vec2(velocity.x, 0.) * time;
            if self.can_move_entity_to_tile(new_pos) {
                entity.set_position(new_pos);
            } else {
                let new_pos = entity.get_position() + vec2(0., velocity.y) * time;
                if self.can_move_entity_to_tile(new_pos) {
                    entity.set_position(new_pos);
                }
            }
        }

        entity.update(&tile.interaction, time);
    }

    fn can_move_entity_to_tile(&self, new_pos: Vec2) -> bool {
        let next_coords = Coords::from_vec2(new_pos);
        match self.get_tile(&next_coords) {
            Some(tile) => match tile.interaction {
                TileInteraction::Block => false,
                TileInteraction::Walkable => true,
                TileInteraction::Swimmable => true,
            },
            None => false,
        }
    }
}

impl Chunk {
    pub fn render(&self) {
        let coords = Coords::from_position(&self.pos);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let tile = &self.tiles[x as usize][y as usize];
                let color = match tile.texture {
                    TileTexture::Grass => GREEN,
                    TileTexture::Water => BLUE,
                    TileTexture::DeepWater => DARKBLUE,
                    TileTexture::Mountain => GRAY,
                    TileTexture::SnowyMountain => WHITE,
                    TileTexture::Sand => YELLOW,
                };
                draw_rectangle((coords.x + x) as f32, (coords.y + y) as f32, 1., 1., color);
            }
        }
        // draw_rectangle_lines(
        //     coords.x as f32,
        //     coords.y as f32,
        //     CHUNK_SIZE as f32,
        //     CHUNK_SIZE as f32,
        //     0.1,
        //     RED,
        // );
    }
    pub fn generate(chunk_pos: ChunkPosition, noise: &Fbm<OpenSimplex>) -> Chunk {
        let mut tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] = Default::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let pos = Coords::from_position_at(&chunk_pos, (x, y)).to_vec2();
                // let n = (noise.get([pos.x / 300., pos.y / 300.]) + 1.) * 0.5;
                let n = (noise.get([(pos.x) as f64, (pos.y) as f64]) + 1.) * 0.5;
                tiles[x as usize][y as usize] = Tile::generate(n);
            }
        }

        return Chunk {
            tiles: tiles,
            pos: chunk_pos,
        };
    }
    pub fn default(chunk_pos: ChunkPosition) -> Chunk {
        let mut tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] = Default::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                tiles[x as usize][y as usize] = Tile::generate(0.5);
            }
        }

        return Chunk {
            tiles: tiles,
            pos: chunk_pos,
        };
    }
    pub fn render_dead(pos: ChunkPosition) {
        let coords = Coords::from_position(&pos).to_vec2();
        draw_rectangle(
            coords.x,
            coords.y,
            CHUNK_SIZE as f32,
            CHUNK_SIZE as f32,
            RED,
        );
    }
}
