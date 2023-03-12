use std::{collections::HashMap, hash::Hash};

use macroquad::prelude::*;

pub trait Entity {
    fn get_velocity(&self) -> Vec2;
    fn get_position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
}

pub struct Player {
    pos: Vec2,
    v: Vec2,
    color: Color,
    textures: HashMap<String, Texture2D>,
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn from_vec2(vec: Vec2) -> Direction {
        if vec.x > 0. && vec.y == 0. {
            Direction::Right
        } else if vec.x < 0. && vec.y == 0. {
            Direction::Left
        } else if vec.x == 0. && vec.y > 0. {
            Direction::Down
        } else if vec.x == 0. && vec.y < 0. {
            Direction::Up
        } else {
            Direction::Right
        }
    }
}

impl Player {
    pub async fn new(x: f32, y: f32) -> Player {
        let textures_names = ["walk down1", "walk left1", "walk right1", "walk up1"];
        let mut textures = HashMap::new();

        for texture_name in textures_names {
            let texture = load_texture(&format!("textures/{}.png",texture_name)).await.unwrap();
            texture.set_filter(FilterMode::Nearest);
            textures.insert(texture_name.to_owned(), texture);
        }
        Player {
            pos: vec2(x, y),
            v: vec2(0., 0.),
            color: PURPLE,
            textures,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    pub fn render(&self) {
        const SIZE: f32 = 1.;

        draw_circle(self.pos.x, self.pos.y, SIZE, self.color);

        let texture = match Direction::from_vec2(self.v) {
            Direction::Up => self.textures.get("walk up1"),
            Direction::Down => self.textures.get("walk down1"),
            Direction::Left => self.textures.get("walk left1"),
            Direction::Right => self.textures.get("walk right1"),
        }.unwrap();
        draw_texture_ex(
            *texture,
            self.pos.x-8.,
            self.pos.y-8.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(16., 16.)),
                ..Default::default()
            },
        );
    }
    pub fn set_velocity(&mut self, velocity: Vec2) {
        self.v = velocity;
    }
}

impl Entity for Player {
    fn get_velocity(&self) -> Vec2 {
        self.v
    }

    fn get_position(&self) -> Vec2 {
        self.pos
    }

    fn set_position(&mut self, pos: Vec2) {
        self.pos = pos;
    }
}
