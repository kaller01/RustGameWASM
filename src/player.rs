use crate::world::TileInteraction;
use macroquad::prelude::*;
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
    textures: &'a HashMap<(Interaction, Direction), Animation>,
    keyframe: KeyFrame,
    keyframe_timer: f32,
    direction: Direction,
    keyframe_interaction: Interaction,
    cooldown: Option<(BlockingAction, f32)>,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum BlockingAction {
    Attack,
    Roll,
    Block,
    Dying,
}

impl BlockingAction {
    pub fn to_interaction(&self) -> Interaction {
        match self {
            BlockingAction::Attack => Interaction::Attack,
            BlockingAction::Roll => Interaction::Roll,
            BlockingAction::Block => Interaction::Block,
            BlockingAction::Dying => Interaction::Dying,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Interaction {
    Swim,
    Walk,
    Idle,
    Attack,
    Roll,
    Block,
    Dying,
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

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
enum KeyFrame {
    Cooldown(u8),
    Free(u8),
}

impl Player<'_> {
    pub fn new_playable<'a>(
        x: f32,
        y: f32,
        textures: &'a HashMap<(Interaction, Direction), Animation>,
    ) -> Player<'a> {
        Player {
            name: "You".to_owned(),
            pos: vec2(x, y),
            v: vec2(0., 0.),
            textures: textures,
            keyframe: KeyFrame::Free(0),
            keyframe_timer: 0.,
            direction: Direction::Right,
            keyframe_interaction: Interaction::Idle,
            cooldown: None,
        }
    }

    pub fn new_other<'a>(
        name: String,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        textures: &'a HashMap<(Interaction, Direction), Animation>,
    ) -> Player<'a> {
        Player {
            name: name,
            pos: vec2(x, y),
            v: vec2(vx, vy),
            textures: textures,
            keyframe: KeyFrame::Free(0),
            keyframe_timer: 0.,
            direction: Direction::Right,
            keyframe_interaction: Interaction::Idle,
            cooldown: None,
        }
    }

    pub fn set_action(&mut self, action: BlockingAction) {
        let velocity = match action {
            BlockingAction::Roll => self.direction.to_vec2() * 30.,
            _ => vec2(0., 0.),
        };

        let cooldown = match action {
            BlockingAction::Attack => 0.8,
            BlockingAction::Roll => 2.,
            BlockingAction::Block => 1.,
            BlockingAction::Dying => 3.,
        };

        self.set_velocity(velocity);
        self.keyframe_interaction = action.to_interaction();
        self.keyframe = KeyFrame::Cooldown(0);
        self.cooldown = Some((action, cooldown));
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn kill(&mut self) {
        self.pos = vec2(-3., -10.);
    }

    pub fn force_action(&mut self, action: BlockingAction, pos: Vec2, direction: Direction) {
        self.pos = pos;
        self.direction = direction;
        self.set_action(action);
    }

    pub fn try_action(&mut self, action: BlockingAction) -> Result<(), ()> {
        match self.keyframe_interaction {
            Interaction::Walk | Interaction::Idle => match self.cooldown {
                Some(_) => Err(()),
                None => match self.keyframe {
                    KeyFrame::Cooldown(_) => Err(()),
                    KeyFrame::Free(_) => {
                        self.set_action(action);
                        Ok(())
                    }
                },
            },
            _ => Err(()),
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

    pub fn render(&mut self, text_params: &TextParams) {
        let texture_data = self
            .textures
            .get(&(self.keyframe_interaction, self.direction))
            .unwrap();
        let texture = texture_data
            .textures
            .get(self.current_keyframe() as usize)
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
        // draw_circle(self.pos.x, self.pos.y, 0.1, GREEN);
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
        let texture_data = self
            .textures
            .get(&(self.keyframe_interaction, self.direction))
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

        match self.keyframe {
            KeyFrame::Cooldown(_) => (),
            KeyFrame::Free(_) => {
                self.keyframe_interaction = match tile_interaction {
                    TileInteraction::Block => Interaction::Idle,
                    TileInteraction::Walkable => Interaction::Walk,
                    TileInteraction::Swimmable => Interaction::Swim,
                };
                match Direction::from_vec2(self.v) {
                    None => {
                        self.keyframe_interaction = match tile_interaction {
                            TileInteraction::Swimmable => Interaction::Swim,
                            _ => Interaction::Idle,
                        }
                    }
                    Some(direction) => self.direction = direction,
                }
            }
        }

        self.keyframe_timer += time;
        match self.cooldown {
            Some((action, cooldown)) => {
                let remaining_cooldown = cooldown - time;
                self.cooldown = Some((action, remaining_cooldown));
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

impl fmt::Display for Interaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Interaction::Swim => write!(f, "swim"),
            Interaction::Walk => write!(f, "walk"),
            Interaction::Idle => write!(f, "idle"),
            Interaction::Attack => write!(f, "attack"),
            Interaction::Roll => write!(f, "roll"),
            Interaction::Block => write!(f, "block"),
            Interaction::Dying => todo!(),
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

pub async fn load_textures() -> HashMap<(Interaction, Direction), Animation> {
    let mut texture_map: HashMap<(Interaction, Direction), Animation> = HashMap::new();

    for action in Interaction::iter() {
        match action {
            Interaction::Dying => {
            },
            _ => {
                for direction in Direction::iter() {
                    let mut textures = Vec::new();
                    let steps = match action {
                        Interaction::Block => vec![""],
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
                        Interaction::Swim => 0.15,
                        Interaction::Walk => 0.3,
                        Interaction::Idle => 0.5,
                        Interaction::Attack => 0.08,
                        Interaction::Roll => 0.2,
                        Interaction::Block => 1.,
                        Interaction::Dying => 1.,
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
        }
    }
    texture_map
}
