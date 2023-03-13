use crate::world::TileInteraction;
use macroquad::{prelude::*};
use std::{collections::HashMap, fmt};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

pub trait Entity {
    fn get_velocity(&self) -> Vec2;
    fn get_position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
    //Should not handle positional changes
    fn update(&mut self, tile: &TileInteraction, time: f32);
}

pub struct Player<'a> {
    name: String,
    pos: Vec2,
    v: Vec2,
    textures: &'a HashMap<(Action, Direction), Animation>,
    keyframe: KeyFrame,
    keyframe_timer: f32,
    direction: Direction,
    current_action: Action,
    cooldown: Option<f32>,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Action {
    Swim,
    Walk,
    Idle,
    Attack,
    Roll,
    Block,
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
    fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => vec2(0., -1.),
            Direction::Down => vec2(0., 1.),
            Direction::Left => vec2(-1., 0.),
            Direction::Right => vec2(1., 0.),
        }
    }
}

enum KeyFrame {
    Cooldown(u8),
    Free(u8),
}

impl Player<'_> {
    pub fn new_playable<'a>(
        x: f32,
        y: f32,
        textures: &'a HashMap<(Action, Direction), Animation>,
    ) -> Player<'a> {
        Player {
            name: "You".to_owned(),
            pos: vec2(x, y),
            v: vec2(0., 0.),
            textures: textures,
            keyframe: KeyFrame::Free(0),
            keyframe_timer: 0.,
            direction: Direction::Right,
            current_action: Action::Idle,
            cooldown: None,
        }
    }

    pub fn new_other<'a>(
        name: String,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        textures: &'a HashMap<(Action, Direction), Animation>,
    ) -> Player<'a> {
        Player {
            name: name,
            pos: vec2(x, y),
            v: vec2(vx, vy),
            textures: textures,
            keyframe: KeyFrame::Free(0),
            keyframe_timer: 0.,
            direction: Direction::Right,
            current_action: Action::Idle,
            cooldown: None,
        }
    }

    fn set_action(&mut self, action: Action, v: Vec2, cooldown: Option<f32>) {
        match self.current_action {
            Action::Walk | Action::Idle => {
                if self.cooldown.is_none() {
                    match self.keyframe {
                        KeyFrame::Cooldown(_) => (),
                        KeyFrame::Free(_) => {
                            self.set_velocity(v);
                            self.current_action = action;
                            self.keyframe = KeyFrame::Cooldown(0);
                            self.cooldown = cooldown
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn advance_keyframe(&mut self) {
        match self.keyframe {
            KeyFrame::Cooldown(frame) => self.keyframe = KeyFrame::Cooldown(frame + 1),
            KeyFrame::Free(frame) => self.keyframe = KeyFrame::Free(frame + 1),
        }
    }

    fn current_keyframe(&mut self) -> u8 {
        match self.keyframe {
            KeyFrame::Cooldown(frame) | KeyFrame::Free(frame) => frame,
        }
    }

    pub fn attack(&mut self) {
        self.set_action(Action::Attack, vec2(0., 0.), Some(0.8));
    }

    pub fn roll(&mut self) {
        self.set_action(Action::Roll, self.direction.to_vec2() * 30., Some(2.))
    }

    pub fn block(&mut self) {
        self.set_action(Action::Block, vec2(0., 0.), None)
    }

    pub fn render(&mut self, text_params: &TextParams) {
        let texture_data = self
            .textures
            .get(&(self.current_action, self.direction))
            .unwrap();
        let texture = texture_data
            .textures
            .get(self.current_keyframe() as usize)
            .unwrap();
        let time_step = texture_data.time_step;
        let steps = texture_data.textures.len();

        match self.keyframe {
            KeyFrame::Cooldown(_frame) | KeyFrame::Free(_frame) => {
                if self.keyframe_timer > time_step {
                    self.advance_keyframe();
                    if self.current_keyframe() == steps as u8 {
                        self.keyframe = KeyFrame::Free(0)
                    }
                    self.keyframe_timer = 0.;
                }
            }
        }

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
        draw_text_ex(&self.name, self.pos.x + 1., self.pos.y - 2., *text_params);
    }
    pub fn set_velocity(&mut self, velocity: Vec2) {
        match self.keyframe {
            KeyFrame::Cooldown(_) => (),
            KeyFrame::Free(_) => self.v = velocity,
        }
    }
}

impl Entity for Player<'_> {
    fn get_velocity(&self) -> Vec2 {
        self.v
    }

    fn get_position(&self) -> Vec2 {
        self.pos
    }

    fn set_position(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn update(&mut self, tile_interaction: &TileInteraction, time: f32) {
        match self.keyframe {
            KeyFrame::Cooldown(_) => (),
            KeyFrame::Free(_) => {
                self.current_action = match tile_interaction {
                    TileInteraction::Block => Action::Idle,
                    TileInteraction::Walkable => Action::Walk,
                    TileInteraction::Swimmable => Action::Swim,
                };
                match Direction::from_vec2(self.v) {
                    None => {
                        self.current_action = match tile_interaction {
                            TileInteraction::Swimmable => Action::Swim,
                            _ => Action::Idle,
                        }
                    }
                    Some(direction) => self.direction = direction,
                }
            }
        }

        self.keyframe_timer += time;
        match self.cooldown {
            Some(cooldown) => {
                println!("{cooldown}");
                let remaining_cooldown = cooldown - time;
                self.cooldown = Some(remaining_cooldown);
                if remaining_cooldown < 0. {
                    self.cooldown = None
                }
            }
            None => (),
        }
    }
}

pub struct Animation {
    time_step: f32,
    textures: Vec<Texture2D>,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Swim => write!(f, "swim"),
            Action::Walk => write!(f, "walk"),
            Action::Idle => write!(f, "idle"),
            Action::Attack => write!(f, "attack"),
            Action::Roll => write!(f, "roll"),
            Action::Block => write!(f, "block"),
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

pub async fn load_textures() -> HashMap<(Action, Direction), Animation> {
    let mut texture_map: HashMap<(Action, Direction), Animation> = HashMap::new();

    for action in Action::iter() {
        for direction in Direction::iter() {
            let mut textures = Vec::new();
            let steps = match action {
                Action::Block => vec![""],
                _ => vec!["1", "2", "3", "4"],
            };
            for step in steps {
                let texture =
                    load_texture(&format!("textures/{} {}{}.png", action, direction, step))
                        .await
                        .unwrap();
                texture.set_filter(FilterMode::Nearest);
                textures.push(texture)
            }

            let time_step = match action {
                Action::Swim => 0.15,
                Action::Walk => 0.3,
                Action::Idle => 0.5,
                Action::Attack => 0.08,
                Action::Roll => 0.2,
                Action::Block => 1.,
            };

            texture_map.insert(
                (action, direction),
                Animation {
                    time_step,
                    textures,
                },
            );
        }
    }
    texture_map
}
