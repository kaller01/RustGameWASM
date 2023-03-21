use macroquad::prelude::*;
use noise::{OpenSimplex, Fbm, NoiseFn};

use super::{tile::Tile, positions::{ChunkPosition, Coords}, CHUNK_SIZE};

#[derive(Debug)]
pub struct Chunk {
    pub tiles: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    pub pos: ChunkPosition,
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

pub struct LazyChunk {
    tile: Tile,
    pos: ChunkPosition,
}

impl LazyChunk {
    pub fn generate(chunk_pos: ChunkPosition, noise: &Fbm<OpenSimplex>) -> LazyChunk {
        let pos = Coords::from_position_at(&chunk_pos, (0, 0)).to_vec2();
        let n = (noise.get([(pos.x) as f64, (pos.y) as f64]) + 1.) * 0.5;
        let tile = Tile::generate(n);
        return LazyChunk {
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