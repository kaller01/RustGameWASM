use macroquad::prelude::Vec2;

#[derive(Debug)]
pub enum Event {
    PlayerUpdate {
        name: String,
        id: u32,
        x: f32,
        y: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    PlayerDisconnect {
        id: u32,
    },
    YourColor {
        r: f32,
        g: f32,
        b: f32,
    },
}

pub trait MultiplayerHandler {
    fn get_events(&self) -> Vec<Event>;
    fn set_your_player_pos(&self, pos: Vec2);
}

pub struct NotImplemented {}

impl MultiplayerHandler for NotImplemented {
    fn get_events(&self) -> Vec<Event> {
        Vec::new()
    }

    fn set_your_player_pos(&self, pos: Vec2) {
        
    }
}