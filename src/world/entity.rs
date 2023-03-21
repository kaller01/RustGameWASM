use macroquad::prelude::*;
use strum_macros::EnumIter;
use super::tile::*;

pub trait WorldEntity {
    fn get_velocity(&self) -> Vec2;
    fn get_position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
    fn get_world_event(&mut self) -> EntityWorldEvent;
    fn give_resource(&mut self, resource: WorldResource);
    //Should not handle positional changes
    fn update(&mut self, tile_interaction: &TileInteraction, tile_action: &TileAction, time: f32);
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum EntityWorldEvent {
    Destroy(Direction),
    Place(WorldResource, Vec2),
    None,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum WorldResource {
    Dirt
}

impl WorldResource {
    pub fn to_tile(&self) -> Tile {
        Tile {
            texture: TileTexture::Dirt,
            interaction: TileInteraction::Block,
            action: TileAction::Destroyable,
        }
    }
}