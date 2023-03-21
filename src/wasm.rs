use macroquad::prelude::*;
use sapp_jsutils::JsObject;
use std::{collections::VecDeque, sync::Mutex};

use crate::multiplayer::Event;
use crate::multiplayer::MultiplayerHandler;
use crate::player::BlockingAction;
use crate::world::entity::Direction;

#[derive(Debug, Clone, Copy)]
struct ThisPlayerData {
    pos: Vec2,
    v: Vec2,
}

impl ThisPlayerData {
    fn new() -> ThisPlayerData {
        ThisPlayerData {
            pos: vec2(0., 0.),
            v: vec2(0., 0.),
        }
    }
}

lazy_static! {
    static ref EVENT_DOWNSTREAM: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
    static ref EVENT_UPSTREAM: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
    static ref THIS_PLAYER: Mutex<ThisPlayerData> = Mutex::new(ThisPlayerData::new());
}

pub struct WasmEventHandler {}

impl MultiplayerHandler for WasmEventHandler {
    fn get_events(&mut self) -> Vec<Event> {
        let size = EVENT_DOWNSTREAM.lock().unwrap().len();
        let events = EVENT_DOWNSTREAM.lock().unwrap().drain(0..size).collect();
        return events;
    }

    fn set_your_player_pos(&self, pos: Vec2, v: Vec2) {
        THIS_PLAYER.lock().unwrap().pos = pos;
        THIS_PLAYER.lock().unwrap().v = v;
    }

    fn upstream_event(&mut self, event: Event) {
        match event {
            Event::PlayerAction {
                id,
                x,
                y,
                direction,
                action,
            } => match action {
                crate::player::BlockingAction::Attack | crate::player::BlockingAction::Roll | crate::player::BlockingAction::Dying  => {
                    EVENT_UPSTREAM.lock().unwrap().push_back(event);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn downstream_event(&mut self, event: Event) {
        EVENT_DOWNSTREAM.lock().unwrap().push_back(event);
    }
}

#[no_mangle]
pub extern "C" fn get_upstream_event() -> JsObject {
    let js_object = JsObject::object();
    let event = EVENT_UPSTREAM.lock().unwrap().pop_front();
    match event {
        Some(event) => match event {
            Event::PlayerAction {
                id,
                x,
                y,
                direction,
                action,
            } => {
                debug!("Upstream {:?}", event);

                let action = match action {
                    BlockingAction::Attack => 1.,
                    BlockingAction::Roll => 2.,
                    BlockingAction::Block => 3.,
                    BlockingAction::Dying => 4.,
                };

                let direction = match direction {
                    crate::world::entity::Direction::Up => 8,
                    crate::world::entity::Direction::Down => 2,
                    crate::world::entity::Direction::Left => 4,
                    crate::world::entity::Direction::Right => 6,
                };

                js_object.set_field_f32("action", action);
                js_object.set_field_f32("direction", direction as f32);
                js_object.set_field_f32("x", x);
                js_object.set_field_f32("y", y);
                js_object
            }
            _ => js_object,
        },
        None => js_object,
    }
}

#[no_mangle]
pub extern "C" fn get_player_update() -> JsObject {
    let player = *THIS_PLAYER.lock().unwrap();
    let js_object = JsObject::object();
    js_object.set_field_f32("x", player.pos.x);
    js_object.set_field_f32("y", player.pos.y);
    js_object.set_field_f32("vx", player.v.x);
    js_object.set_field_f32("vy", player.v.y);
    js_object
}

#[no_mangle]
pub extern "C" fn update_player(js_object: JsObject, id: u32, x: f32, y: f32, vx: f32, vy: f32) {
    let mut name = String::new();
    js_object.to_string(&mut name);

    let event = Event::PlayerUpdate {
        name,
        id,
        x,
        y,
        vx,
        vy,
    };
    EVENT_DOWNSTREAM.lock().unwrap().push_back(event);
}

#[no_mangle]
pub extern "C" fn teleport(x: f32, y:f32) {
    let event = Event::CommandTeleport {
        x,
        y
    };
    EVENT_DOWNSTREAM.lock().unwrap().push_back(event);
}

#[no_mangle]
pub extern "C" fn downstream_player_action(id: u32, x: f32, y: f32, action: f32, direction: f32) {
    let mut d = Direction::Up;
    let mut a = BlockingAction::Attack;

    if direction == 8. {
        d = Direction::Up;
    } else if direction == 2. {
        d = Direction::Down;
    } else if direction == 4. {
        d = Direction::Left;
    } else if direction == 6. {
        d = Direction::Right;
    }

    if action == 1. {
        a = BlockingAction::Attack;
    } else if action == 2. {
        a = BlockingAction::Roll;
    } else if action == 4. {
        a = BlockingAction::Dying;
    }

    let event = Event::PlayerAction {
        id,
        x,
        y,
        direction: d,
        action: a,
    };

    debug!("Downstream {:?}", event);

    EVENT_DOWNSTREAM.lock().unwrap().push_back(event);
}

#[no_mangle]
pub extern "C" fn disconnect_player(id: u32) {
    let event = Event::PlayerDisconnect { id };
    EVENT_DOWNSTREAM.lock().unwrap().push_back(event);
}
