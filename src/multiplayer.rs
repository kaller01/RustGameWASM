use macroquad::prelude::Vec2;

use crate::player::{BlockingAction, Direction};

#[derive(Debug)]
pub enum Event {
    PlayerUpdate {
        name: String,
        id: u32,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
    },
    PlayerAction {
        id: u32,
        x: f32,
        y: f32,
        direction: Direction,
        action: BlockingAction
    },
    PlayerDisconnect {
        id: u32,
    },
    CommandTeleport {
        x: f32,
        y: f32
    }
}

pub trait MultiplayerHandler {
    fn get_events(&mut self) -> Vec<Event>;
    fn upstream_event(&mut self, event: Event);
    fn set_your_player_pos(&self, pos: Vec2, v: Vec2);
    fn downstream_event(&mut self, event: Event); //Only necessary for local
}

pub struct DevLocalMultiplayer {
    events: Vec<Event>
}

impl DevLocalMultiplayer {
    pub fn new() -> DevLocalMultiplayer{
        DevLocalMultiplayer {
            events: Vec::new()
        }
    }
}

impl MultiplayerHandler for DevLocalMultiplayer {
    fn get_events(&mut self) -> Vec<Event> {
        let size = self.events.len();
        let events = self.events.drain(0..size).collect();
        return events;
    }

    fn set_your_player_pos(&self, _pos: Vec2, _v: Vec2) {
        
    }

    fn upstream_event(&mut self, _event: Event) {
        
    }

    fn downstream_event(&mut self, event: Event) {
        self.events.push(event)
    }
}