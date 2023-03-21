use std::collections::HashMap;

use macroquad::prelude::*;
use noise::{OpenSimplex, Fbm, MultiFractal};

use self::{positions::*, chunk::*, tile::*, entity::*};

pub mod chunk;
pub mod tile;
pub mod positions;
pub mod entity;

const CHUNK_SIZE: i32 = 8;

pub struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
    map_chunks: HashMap<ChunkPosition, LazyChunk>,
    noise: Fbm<OpenSimplex>,
}

impl World {
    pub fn generate() -> World {
        let noise = Fbm::<OpenSimplex>::new(0)
            .set_frequency(0.01)
            .set_persistence(0.6)
            .set_lacunarity(2.)
            .set_octaves(5);

        let x1 = -10;
        let y1 = -10;
        let x2 = 10;
        let y2 = 10;

        let mut chunks: HashMap<ChunkPosition, Chunk> = HashMap::new();
        let mut map_chunks: HashMap<ChunkPosition, LazyChunk> = HashMap::new();

        for x in x1..x2 {
            for y in y1..y2 {
                let pos = ChunkPosition { x, y };
                let chunk = Chunk::generate(pos, &noise);
                chunks.insert(pos, chunk);
                let map_chunk = LazyChunk::generate(pos, &noise);
                map_chunks.insert(pos, map_chunk);
            }
        }
        return World {
            chunks,
            noise,
            map_chunks,
        };
    }

    pub fn render_map(&mut self, view: Rect) {
        for x in (view.x as i32)..(view.x + view.w) as i32 {
            for y in (view.y as i32)..(view.y + view.h) as i32 {
                let pos = ChunkPosition { x, y };
                match (self.map_chunks.get(&pos), self.chunks.get(&pos)) {
                    (None, _) => {
                        draw_rectangle(
                            pos.x as f32,
                            pos.y as f32,
                            1.,
                            1.,
                            color_u8!(100, 100, 100, 100),
                        );
                    }
                    (Some(map), None) => map.render_small(false),
                    (Some(map), Some(_)) => map.render_small(true),
                }
            }
        }
    }

    pub fn render(&self, rect: Rect) {
        let area = ChunkPosition::from_rect(rect);

        for x in area.0.x..area.1.x {
            for y in area.0.y..area.1.y {
                let pos = ChunkPosition { x, y };
                match self.chunks.get(&pos) {
                    Some(chunk) => chunk.render(),
                    None => match self.map_chunks.get(&pos) {
                        Some(chunk) => chunk.render(),
                        None => (),
                    },
                }
            }
        }
    }
    pub fn generate_at(&mut self, render_zone: Rect, map_zone: Rect) {
        let area = ChunkPosition::from_rect(render_zone);
        for x in area.0.x..area.1.x {
            for y in area.0.y..area.1.y {
                let pos = ChunkPosition { x, y };
                if !self.chunks.contains_key(&pos) {
                    let chunk = Chunk::generate(pos, &self.noise);
                    self.chunks.insert(pos, chunk);
                }
            }
        }
        self.generate_map_at(map_zone);
    }

    fn generate_map_at(&mut self, map_zone: Rect) {
        let area = ChunkPosition::from_rect(map_zone);
        for x in area.0.x..area.1.x {
            for y in area.0.y..area.1.y {
                let pos = ChunkPosition { x, y };
                if !self.map_chunks.contains_key(&pos) {
                    let chunk = LazyChunk::generate(pos, &self.noise);
                    self.map_chunks.insert(pos, chunk);
                }
            }
        }
    }
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

    fn set_tile(&mut self, coords: &Coords, tile: Tile) {
        let chunk_pos = ChunkPosition::from_coords(coords);
        let index = (
            (coords.x.rem_euclid(CHUNK_SIZE) as usize),
            (coords.y.rem_euclid(CHUNK_SIZE) as usize),
        );
        match self.chunks.get_mut(&chunk_pos) {
            Some(chunk) => {
                chunk.tiles[index.0][index.1] = tile;
            }
            None => (),
        }
    }

    pub fn update_entity(&self, entity: &mut dyn WorldEntity, time: f32) {
        let current_coords = Coords::from_vec2(entity.get_position());
        let mut velocity = entity.get_velocity();

        match self.get_tile(&current_coords) {
            Some(tile) => {
                match tile.interaction {
                    TileInteraction::Swimmable => velocity *= 0.6,
                    TileInteraction::Crawl => velocity *= 0.4,
                    _ => (),
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

                entity.update(&tile.interaction, &tile.action, time);
            }
            None => (),
        };
    }

    pub fn update_world_by_entity(&mut self, entity: &mut dyn WorldEntity) {
        let world_event = entity.get_world_event();
        match world_event {
            EntityWorldEvent::Destroy(direction) => {
                let current_coords = Coords::from_vec2(entity.get_position());

                let tiles_coords = get_tiles_in_half_circle(current_coords, direction, 3.);
                for tile_coords in tiles_coords {
                    match self.get_tile(&tile_coords) {
                        Some(tile) => match tile.action {
                            TileAction::Destroyable => self.set_tile(
                                &tile_coords,
                                Tile {
                                    texture: TileTexture::Grass,
                                    interaction: TileInteraction::Walkable,
                                    action: TileAction::None,
                                },
                            ),
                            _ => (),
                        },
                        None => (),
                    }
                }
            }
            _ => (),
        }
    }

    fn can_move_entity_to_tile(&self, new_pos: Vec2) -> bool {
        let next_coords = Coords::from_vec2(new_pos);
        match self.get_tile(&next_coords) {
            Some(tile) => match tile.interaction {
                TileInteraction::Block => false,
                TileInteraction::Walkable => true,
                TileInteraction::Swimmable => true,
                TileInteraction::Crawl => true,
            },
            None => false,
        }
    }
}
