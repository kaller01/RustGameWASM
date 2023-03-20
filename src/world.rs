use crate::player::{Direction, Entity};
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
    Crawl,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileAction {
    None,
    Destroyable,
    Death,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileTexture {
    Grass,
    Water,
    ShallowWater,
    Sand,
    DeepWater,
    Dirt,
    Stone,
    SnowyMountain,
}

impl TileTexture {
    fn to_color(&self) -> Color {
        match self {
            TileTexture::Grass => GREEN,
            TileTexture::ShallowWater => SKYBLUE,
            TileTexture::Water => BLUE,
            TileTexture::DeepWater => DARKBLUE,
            TileTexture::Stone => GRAY,
            TileTexture::SnowyMountain => WHITE,
            TileTexture::Sand => YELLOW,
            TileTexture::Dirt => Color::from_rgba(155, 118, 83, 255),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Tile {
    texture: TileTexture,
    interaction: TileInteraction,
    action: TileAction,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            texture: TileTexture::Grass,
            interaction: TileInteraction::Walkable,
            action: TileAction::None,
        }
    }
}

impl Tile {
    fn generate(n: f64) -> Tile {
        let (texture, interaction, action) = if n < 0.25 {
            (
                TileTexture::DeepWater,
                TileInteraction::Walkable,
                TileAction::Death,
            )
        } else if n < 0.43 {
            (
                TileTexture::Water,
                TileInteraction::Swimmable,
                TileAction::None,
            )
        } else if n < 0.5 {
            (
                TileTexture::ShallowWater,
                TileInteraction::Crawl,
                TileAction::None,
            )
        } else if n < 0.52 {
            (
                TileTexture::Sand,
                TileInteraction::Walkable,
                TileAction::None,
            )
        } else if n < 0.7 {
            (
                TileTexture::Grass,
                TileInteraction::Walkable,
                TileAction::None,
            )
        } else if n < 0.72 {
            (
                TileTexture::Dirt,
                TileInteraction::Block,
                TileAction::Destroyable,
            )
        } else if n < 0.85 {
            (TileTexture::Stone, TileInteraction::Block, TileAction::Death)
        } else {
            (
                TileTexture::SnowyMountain,
                TileInteraction::Block,
                TileAction::Death,
            )
        };
        Tile {
            texture,
            interaction,
            action,
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
    map_chunks: HashMap<ChunkPosition, MapChunk>,
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
        let mut map_chunks: HashMap<ChunkPosition, MapChunk> = HashMap::new();

        for x in x1..x2 {
            for y in y1..y2 {
                let pos = ChunkPosition { x, y };
                let chunk = Chunk::generate(pos, &noise);
                chunks.insert(pos, chunk);
                let map_chunk = MapChunk::generate(pos, &noise);
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
                    let chunk = MapChunk::generate(pos, &self.noise);
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

    pub fn update_entity(&self, entity: &mut dyn Entity, time: f32) {
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

    pub fn update_world_by_entity(&mut self, entity: &mut dyn Entity) {
        let world_event = entity.get_world_event();
        match world_event {
            crate::player::WorldEvent::Destroy(direction) => {
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

impl Chunk {
    pub fn render_border(&self) {
        let coords = Coords::from_position(&self.pos);
        draw_rectangle_lines(
            coords.x as f32,
            coords.y as f32,
            CHUNK_SIZE as f32,
            CHUNK_SIZE as f32,
            0.1,
            RED,
        );
    }

    pub fn render(&self) {
        let coords = Coords::from_position(&self.pos);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let tile = &self.tiles[x as usize][y as usize];
                let color = tile.texture.to_color();
                draw_rectangle((coords.x + x) as f32, (coords.y + y) as f32, 1., 1., color);
            }
        }
    }

    pub fn render_lazy(&self) {}

    pub fn generate(chunk_pos: ChunkPosition, noise: &Fbm<OpenSimplex>) -> Chunk {
        let mut tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] = Default::default();

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let pos = Coords::from_position_at(&chunk_pos, (x, y)).to_vec2();
                let n = (noise.get([(pos.x) as f64, (pos.y) as f64]) + 1.) * 0.5;
                tiles[x as usize][y as usize] = Tile::generate(n);
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

struct MapChunk {
    tile: Tile,
    pos: ChunkPosition,
}

impl MapChunk {
    pub fn generate(chunk_pos: ChunkPosition, noise: &Fbm<OpenSimplex>) -> MapChunk {
        let pos = Coords::from_position_at(&chunk_pos, (0, 0)).to_vec2();
        let n = (noise.get([(pos.x) as f64, (pos.y) as f64]) + 1.) * 0.5;
        let tile = Tile::generate(n);
        return MapChunk {
            tile: tile,
            pos: chunk_pos,
        };
    }

    pub fn render_small(&self, discovered: bool) {
        let color = self.tile.texture.to_color();
        draw_rectangle(
            self.pos.x as f32,
            self.pos.y as f32,
            1. as f32,
            1. as f32,
            color,
        );
        if !discovered {
            draw_rectangle(
                self.pos.x as f32,
                self.pos.y as f32,
                1.,
                1.,
                color_u8!(100, 100, 100, 100),
            );
        }
    }

    pub fn render(&self) {
        let coords = Coords::from_position(&self.pos);
        let color = self.tile.texture.to_color();
        draw_rectangle(
            (coords.x) as f32,
            (coords.y) as f32,
            CHUNK_SIZE as f32,
            CHUNK_SIZE as f32,
            color,
        );
        draw_rectangle(
            (coords.x) as f32,
            (coords.y) as f32,
            CHUNK_SIZE as f32,
            CHUNK_SIZE as f32,
            color_u8!(100, 100, 100, 100),
        );
    }
}

pub fn get_tiles_in_half_circle(center: Coords, direction: Direction, radius: f32) -> Vec<Coords> {
    let mut tiles = Vec::new();

    let facing_vec = direction.to_vec2();
    let facing_angle = facing_vec.y.atan2(facing_vec.x);
    let half_circle_angle = std::f32::consts::PI / 2.0;

    for x in (center.x - radius as i32)..=(center.x + radius as i32) {
        for y in (center.y - radius as i32)..=(center.y + radius as i32) {
            let tile_coords = Coords { x, y };

            // Check if tile is within radius
            let distance_squared =
                (tile_coords.x - center.x).pow(2) + (tile_coords.y - center.y).pow(2);
            let radius_squared = radius.powf(2.);
            if distance_squared as f32 <= radius_squared {
                // Check if tile is within half circle facing in player direction
                let tile_vec = Vec2 {
                    x: (tile_coords.x - center.x) as f32,
                    y: (tile_coords.y - center.y) as f32,
                };
                let tile_angle = tile_vec.y.atan2(tile_vec.x);
                let angle_diff = (tile_angle - facing_angle).abs();
                let within_half_circle = angle_diff <= half_circle_angle;
                if within_half_circle {
                    tiles.push(tile_coords);
                }
            }
        }
    }

    tiles
}
