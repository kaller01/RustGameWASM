use macroquad::prelude::Vec2;

#[derive(Debug)]
pub enum Event {
    PlayerUpdate {
        name: String,
        id: u32,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32
    },
    PlayerDisconnect {
        id: u32,
    },
}

pub trait MultiplayerHandler {
    fn get_events(&mut self) -> Vec<Event>;
    fn add_event(&mut self, event: Event);
    fn set_your_player_pos(&self, pos: Vec2, v: Vec2);
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

    fn add_event(&mut self, event: Event) {
        self.events.push(event)
    }
}