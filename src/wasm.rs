use macroquad::prelude::*;
use sapp_jsutils::JsObject;
use std::{collections::VecDeque, sync::Mutex};

use crate::multiplayer::Event;
use crate::multiplayer::MultiplayerHandler;

lazy_static! {
    static ref EVENT_BUFFER: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
    static ref THIS_PLAYER_POS: Mutex<Vec2> = Mutex::new(vec2(0., 0.));
    static ref THIS_PLAYER_VELOCITY: Mutex<Vec2> = Mutex::new(vec2(0., 0.));
}

pub struct WasmEventHandler {}

impl MultiplayerHandler for WasmEventHandler {
    fn get_events(&self) -> Vec<Event> {
        let size = EVENT_BUFFER.lock().unwrap().len();
        let events = EVENT_BUFFER.lock().unwrap().drain(0..size).collect();
        return events;
    }

    fn set_your_player_pos(&self, pos: Vec2, v: Vec2) {
        THIS_PLAYER_POS.lock().unwrap().x = pos.x;
        THIS_PLAYER_POS.lock().unwrap().y = pos.y;
        THIS_PLAYER_VELOCITY.lock().unwrap().x = v.x;
        THIS_PLAYER_VELOCITY.lock().unwrap().y = v.y;
    }
}

#[no_mangle]
pub extern "C" fn get_player_pos_x() -> f32 {
    let pos = THIS_PLAYER_POS.lock().unwrap();
    return pos.x;
}

#[no_mangle]
pub extern "C" fn get_player_pos_y() -> f32 {
    let pos = THIS_PLAYER_POS.lock().unwrap();
    return pos.y;
}

#[no_mangle]
pub extern "C" fn get_player_v_x() -> f32 {
    let pos = THIS_PLAYER_VELOCITY.lock().unwrap();
    return pos.x;
}

#[no_mangle]
pub extern "C" fn get_player_v_y() -> f32 {
    let pos = THIS_PLAYER_VELOCITY.lock().unwrap();
    return pos.y;
}


#[no_mangle]
pub extern "C" fn update_player(
    js_object: JsObject,
    id: u32,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32
) {
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
    // debug!("{:?}", event);
    EVENT_BUFFER.lock().unwrap().push_back(event);
}

#[no_mangle]
pub extern "C" fn disconnect_player(id: u32) {
    let event = Event::PlayerDisconnect { id };
    EVENT_BUFFER.lock().unwrap().push_back(event);
}

// #[no_mangle]
// pub extern "C" fn your_color(r: f32, g: f32, b: f32) {
//     let event = Event::YourColor { r, g, b };
//     EVENT_BUFFER.lock().unwrap().push_back(event);
// }

// #[no_mangle]
// pub extern "C" fn hi_rust_with_struct(js_object: JsObject) {
//     let mut data = String::new();

//     let foo = js_object.field("foo");
//     foo.to_string(&mut data);
//     miniquad::debug!("{}", &data);

//     let foo = js_object.field_f32("bar");
//     miniquad::debug!("{}", foo);
// }

// #[no_mangle]
// extern "C" {
//     fn perform_demo() -> f32;
// }

// #[no_mangle]
// pub extern "C" fn test() {
//     let js_object: f32 = unsafe { perform_demo() };
//     debug!("hello!");
//     // let mut message = String::new();

//     // js_object.to_string(&mut message);

//     debug!("{}", js_object);
// }
