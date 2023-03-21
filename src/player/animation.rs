use crate::world::{entity::*};
use macroquad::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, hash::Hash};
use strum::IntoEnumIterator; // 0.17.1

use super::{BlockingAction, Interaction}; // 0.17.1


#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum KeyFrame {
    Blocking(u8, BlockingAction),
    Free(u8, Interaction),
}

pub struct Animation {
    pub texture_pos: Vec<PixelPos>,
    pub time_step: f32,
}

impl fmt::Display for Interaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interaction::Swim => write!(f, "swim"),
            Interaction::Walk => write!(f, "walk"),
            Interaction::Idle => write!(f, "idle"),
            Interaction::Attack => write!(f, "attack"),
            Interaction::Roll => write!(f, "roll"),
            Interaction::Block => write!(f, "block"),
            Interaction::Dying => write!(f, "death"),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Spritesheet {
    #[serde(rename = "frames")]
    pub frames: HashMap<String, Frame>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Frame {
    #[serde(rename = "frame")]
    pub pos: PixelPos,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PixelPos {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

pub async fn load_textures() -> (Texture2D, HashMap<(Interaction, Direction), Animation>) {
    let mut texture_map: HashMap<(Interaction, Direction), Animation> = HashMap::new();

    let texture = load_texture("textures/spritesheet.png").await.unwrap();
    texture.set_filter(FilterMode::Nearest);
    let spritesheet = load_string("textures/spritesheet.json").await.unwrap();
    let spritesheet: Spritesheet = serde_json::from_str(&spritesheet).unwrap();

    for action in Interaction::iter() {
        match action {
            Interaction::Dying => {
                for direction in Direction::iter() {
                    let mut textures = Vec::new();
                    for step in vec!["1", "2", "3", "4"] {
                        let frame = spritesheet
                            .frames
                            .get(&format!("{}{}.png", action, step))
                            .unwrap();
                        textures.push(frame.pos);
                    }
                    texture_map.insert(
                        (action, direction),
                        Animation {
                            texture_pos: textures,
                            time_step: 0.3,
                        },
                    );
                }
            }
            _ => {
                for direction in Direction::iter() {
                    let mut textures = Vec::new();
                    let steps = match action {
                        Interaction::Block => vec![""],
                        _ => vec!["1", "2", "3", "4"],
                    };
                    for step in steps {
                        let frame = spritesheet
                            .frames
                            .get(&format!("{} {}{}.png", action, direction, step))
                            .unwrap();
                        textures.push(frame.pos);
                    }

                    let time_step = match action {
                        Interaction::Swim => 0.15,
                        Interaction::Walk => 0.15,
                        Interaction::Idle => 0.5,
                        Interaction::Attack => 0.08,
                        Interaction::Roll => 0.2,
                        Interaction::Block => 1.,
                        Interaction::Dying => 1.,
                    };

                    texture_map.insert(
                        (action, direction),
                        Animation {
                            texture_pos: textures,
                            time_step,
                        },
                    );
                }
            }
        }
    }

    (texture, texture_map)
}
