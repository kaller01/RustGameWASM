use std::collections::HashMap;

use macroquad::prelude::*;
use strum_macros::EnumIter;

use crate::world::entity::{Direction, EntityWorldEvent, WorldResource};

use self::{animation::*};

pub mod player;
pub mod animation;

const ATTACK_COOLDOWN: f32 = 1.0;

pub struct Player<'a> {
    name: String,
    pos: Vec2,
    v: Vec2,
    animations: &'a HashMap<(Interaction, Direction), Animation>,
    textures: &'a Texture2D,
    keyframe: KeyFrame,
    keyframe_timer: f32,
    direction: Direction,
    cooldowns: HashMap<BlockingAction, f32>,
    world_events: Vec<EntityWorldEvent>,
    local_player: bool,
    resources: HashMap<WorldResource, u32>
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum BlockingAction {
    Attack,
    Roll,
    Block,
    Dying,
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