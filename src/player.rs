use crate::world::{Tile, TileInteraction};
use macroquad::prelude::*;
use std::{collections::HashMap, hash::Hash};

pub trait Entity {
    fn get_velocity(&self) -> Vec2;
    fn get_position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
    //Should not handle positional changes
    fn notify(&mut self, tile: &TileInteraction, time: f32);
}

pub struct Player {
    pos: Vec2,
    v: Vec2,
    color: Color,
    textures: HashMap<String, Texture2D>,
    walk_step: u8,
    time_to_next_step: f32,
    last_direction: Direction,
    current_interaction: Interaction,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Interaction {
    Swim,
    Walk,
    Idle,
}

impl Direction {
    fn from_vec2(vec: Vec2) -> Option<Direction> {
        if vec.x > 0. && vec.y == 0. {
            Some(Direction::Right)
        } else if vec.x < 0. && vec.y == 0. {
            Some(Direction::Left)
        } else if vec.x == 0. && vec.y > 0. {
            Some(Direction::Down)
        } else if vec.x == 0. && vec.y < 0. {
            Some(Direction::Up)
        } else if vec.x > 0. {
            Some(Direction::Right)
        } else if vec.x < 0. {
            Some(Direction::Left)
        } else {
            None
        }
    }
}

impl Player {
    pub async fn new(x: f32, y: f32) -> Player {
        let mut textures = HashMap::new();
        let mut textures_names: Vec<String> = Vec::new();

        for action in ["walk", "swim", "idle"] {
            for direction in ["down", "left", "right", "up"] {
                for step in ["1", "2", "3", "4"] {
                    textures_names.push(format!("{} {}{}", action, direction, step).to_owned())
                }
            }
        }

        for texture_name in textures_names {
            let texture = load_texture(&format!("textures/{}.png", texture_name))
                .await
                .unwrap();
            texture.set_filter(FilterMode::Nearest);
            textures.insert(texture_name.to_owned(), texture);
        }
        Player {
            pos: vec2(x, y),
            v: vec2(0., 0.),
            color: PURPLE,
            textures,
            walk_step: 1,
            time_to_next_step: 0.,
            last_direction: Direction::Right,
            current_interaction: Interaction::Idle,
        }
    }
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn render(&self) {
        let action = match self.current_interaction {
            Interaction::Swim => "swim",
            Interaction::Walk => "walk",
            Interaction::Idle => "idle",
        };

        let texture = match self.last_direction {
            Direction::Up => self.textures.get(&format!(
                "{} {}{}",
                action,
                "up",
                self.walk_step.to_string()
            )),
            Direction::Down => self.textures.get(&format!(
                "{} {}{}",
                action,
                "down",
                self.walk_step.to_string()
            )),
            Direction::Left => self.textures.get(&format!(
                "{} {}{}",
                action,
                "left",
                self.walk_step.to_string()
            )),
            Direction::Right => self.textures.get(&format!(
                "{} {}{}",
                action,
                "right",
                self.walk_step.to_string()
            )),
        }
        .unwrap();
        draw_texture_ex(
            *texture,
            self.pos.x - 8.,
            self.pos.y - 8.,
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

    fn notify(&mut self, tile_interaction: &TileInteraction, time: f32) {
        self.current_interaction = match tile_interaction {
            TileInteraction::Block => Interaction::Idle,
            TileInteraction::Walkable => Interaction::Walk,
            TileInteraction::Swimmable => Interaction::Swim,
        };
        match Direction::from_vec2(self.v) {
            None => self.current_interaction = Interaction::Idle,
            Some(direction) => self.last_direction = direction,
        }
        self.time_to_next_step += time;
        let time_step = match self.current_interaction {
            Interaction::Swim => 0.3,
            Interaction::Walk => 0.15,
            Interaction::Idle => 0.3,
        };
        if self.time_to_next_step > time_step {
            self.walk_step += 1;
            if self.walk_step == 5 {
                self.walk_step = 1
            }

            self.time_to_next_step = 0.;
        }
    }
}
