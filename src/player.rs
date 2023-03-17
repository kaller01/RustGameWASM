use crate::world::TileInteraction;
use macroquad::prelude::*;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, hash::Hash};
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
    animations: &'a HashMap<(Interaction, Direction), Animation>,
    textures: &'a Texture2D,
    keyframe: KeyFrame,
    keyframe_timer: f32,
    direction: Direction,
    // keyframe_interaction: Interaction,
    cooldowns: HashMap<BlockingAction, f32>,
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

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
enum KeyFrame {
    Blocking(u8, BlockingAction),
    Free(u8, Interaction),
}

impl Player<'_> {
    pub fn new_playable<'a>(
        x: f32,
        y: f32,
        animations: &'a HashMap<(Interaction, Direction), Animation>,
        textures: &'a Texture2D,
    ) -> Player<'a> {
        Player {
            name: "You".to_owned(),
            pos: vec2(x, y),
            v: vec2(0., 0.),
            animations,
            textures,
            keyframe: KeyFrame::Free(0, Interaction::Idle),
            keyframe_timer: 0.,
            direction: Direction::Right,
            cooldowns: HashMap::new(),
        }
    }

    pub fn new_other<'a>(
        name: String,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        animations: &'a HashMap<(Interaction, Direction), Animation>,
        textures: &'a Texture2D,
    ) -> Player<'a> {
        Player {
            name,
            pos: vec2(x, y),
            v: vec2(vx, vy),
            animations,
            textures,
            keyframe: KeyFrame::Free(0, Interaction::Idle),
            keyframe_timer: 0.,
            direction: Direction::Right,
            cooldowns: HashMap::new(),
        }
    }

    pub fn set_action(&mut self, action: BlockingAction) {
        let velocity = match action {
            BlockingAction::Roll => self.direction.to_vec2() * 30.,
            _ => vec2(0., 0.),
        };

        self.set_velocity(velocity);
        self.keyframe = KeyFrame::Blocking(0, action);
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn kill(&mut self) {
        self.set_action(BlockingAction::Dying)
    }

    fn respawn(&mut self) {
        let spawn_points: Vec<Vec2> = vec![vec2(-40., -20.), vec2(5., -30.), vec2(-15., -10.)];
        let index = rand::gen_range(0, spawn_points.len());
        self.pos = *spawn_points.get(index).unwrap();
    }

    //used for multiplayer
    pub fn force_action(&mut self, action: BlockingAction, pos: Vec2, direction: Direction) {
        self.pos = pos;
        self.direction = direction;
        self.set_action(action);
    }

    pub fn try_action(&mut self, action: BlockingAction) -> Result<(), ()> {
        match self.keyframe {
            KeyFrame::Blocking(_, _) => Err(()),
            KeyFrame::Free(_, interaction) => match interaction {
                Interaction::Walk | Interaction::Idle => match self.cooldowns.get(&action) {
                    Some(_) => Err(()),
                    None => {
                        self.set_action(action);
                        Ok(())
                    }
                },
                _ => Err(()),
            },
        }
    }

    fn advance_keyframe(&mut self) {
        match self.keyframe {
            KeyFrame::Blocking(frame, action) => {
                self.keyframe = KeyFrame::Blocking(frame + 1, action)
            }
            KeyFrame::Free(frame, interaction) => {
                self.keyframe = KeyFrame::Free(frame + 1, interaction)
            }
        }
    }

    fn current_keyframe(&mut self) -> u8 {
        match self.keyframe {
            KeyFrame::Blocking(frame, _) | KeyFrame::Free(frame, _) => frame,
        }
    }

    //Called when animation is changing
    fn free_keyframe(&mut self, next_interaction: Interaction) {
        match self.keyframe {
            KeyFrame::Blocking(_, action) => match action {
                BlockingAction::Attack => {
                    self.cooldowns.insert(action, 0.3);
                }
                BlockingAction::Roll => {
                    self.cooldowns.insert(action, 1.);
                }
                BlockingAction::Dying => self.respawn(),
                _ => (),
            },
            KeyFrame::Free(_, _) => {}
        }
        self.keyframe = KeyFrame::Free(0, next_interaction);
    }

    //called when cooldown is reset.
    fn reset_cooldown(&mut self, from_action: BlockingAction) {
        match from_action {
            BlockingAction::Attack => (),
            BlockingAction::Roll => (),
            BlockingAction::Block => (),
            BlockingAction::Dying => self.set_position(vec2(0., 0.)),
        }
        self.cooldowns.remove(&from_action);
    }

    pub fn render(&mut self, text_params: &TextParams, debug: bool) {
        let interaction = match self.keyframe {
            KeyFrame::Blocking(_, action) => action.to_interaction(),
            KeyFrame::Free(_, interaction) => interaction,
        };

        match self.animations.get(&(interaction, self.direction)) {
            Some(texture_data) => {
                let pixel_pos = texture_data
                    .texture_pos
                    .get(self.current_keyframe() as usize)
                    .unwrap();

                

                draw_texture_ex(
                    *self.textures,
                    self.pos.x - 8.,
                    self.pos.y - 8.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(16., 16.)),
                        source: Some(Rect::new(
                            pixel_pos.x,
                            pixel_pos.y,
                            pixel_pos.w,
                            pixel_pos.h,
                        )),
                        ..Default::default()
                    },
                );
            }
            None => {
                draw_circle(self.pos.x, self.pos.y, 1., RED);
            }
        }
        if debug {
            draw_circle(self.pos.x, self.pos.y, 0.1, RED);
            draw_circle_lines(self.pos.x,self.pos.y,2.5,0.1,RED);
        }

        draw_text_ex(&self.name, self.pos.x + 1., self.pos.y - 2., *text_params);
    }
    pub fn set_velocity(&mut self, velocity: Vec2) {
        match self.keyframe {
            KeyFrame::Blocking(_, _) => (),
            KeyFrame::Free(_, _) => self.v = velocity,
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
        let next_possible_interaction = match tile_interaction {
            TileInteraction::Block => Interaction::Idle,
            TileInteraction::Walkable => Interaction::Walk,
            TileInteraction::Swimmable => Interaction::Swim,
            TileInteraction::Crawl => Interaction::Walk,
        };

        let interaction = match self.keyframe {
            KeyFrame::Blocking(_, action) => action.to_interaction(),
            KeyFrame::Free(_, interaction) => interaction,
        };

        {
            //Keyframe timer
            match self.animations.get(&(interaction, self.direction)) {
                Some(texture_data) => {
                    let time_step = texture_data.time_step;
                    let steps = texture_data.texture_pos.len();
                    if self.keyframe_timer > time_step {
                        self.advance_keyframe();
                        if self.current_keyframe() == steps as u8 {
                            match self.keyframe {
                                //Keyframe loop complete
                                KeyFrame::Blocking(_, _) => {
                                    self.free_keyframe(next_possible_interaction)
                                }
                                KeyFrame::Free(_, interaction) => {
                                    if interaction != next_possible_interaction {
                                        self.free_keyframe(next_possible_interaction);
                                    } else {
                                        //Keyframe looping
                                        self.keyframe = KeyFrame::Free(0, interaction);
                                    }
                                }
                            }
                        } else {
                            //If keyframe is free, change animation mid keyframe
                            match self.keyframe {
                                KeyFrame::Blocking(_, _) => (),
                                KeyFrame::Free(_, interaction) => {
                                    if interaction != next_possible_interaction {
                                        self.free_keyframe(next_possible_interaction)
                                    }
                                }
                            }
                        }
                        self.keyframe_timer = 0.;
                    }
                    self.keyframe_timer += time;
                }
                None => {}
            }
        }

        //Direction is allowed to be changed if the keyframe is free
        match self.keyframe {
            KeyFrame::Blocking(_, _) => (),
            KeyFrame::Free(_, _) => match Direction::from_vec2(self.v) {
                None => {}
                Some(direction) => self.direction = direction,
            },
        }

        //Cooldown timer
        let mut to_be_removed = Vec::new();
        for (action, cooldown) in self.cooldowns.iter_mut() {
            let remaining_cooldown = *cooldown - time;
            if remaining_cooldown < 0. {
                to_be_removed.push(action.clone());
            } else {
                *cooldown = remaining_cooldown;
            }
        }
        for to_be_removed in to_be_removed {
            self.reset_cooldown(to_be_removed);
        }
    }
}

pub struct Animation {
    texture_pos: Vec<PixelPos>,
    time_step: f32,
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
    frames: HashMap<String, Frame>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Frame {
    #[serde(rename = "frame")]
    pos: PixelPos,
}

#[derive(Debug, Deserialize, Serialize,Clone, Copy)]
struct PixelPos {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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
                        let frame = spritesheet.frames.get(&format!("{}{}.png", action, step)).unwrap();
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
                        let frame = spritesheet.frames.get(&format!("{} {}{}.png", action, direction, step)).unwrap();
                        textures.push(frame.pos);
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
