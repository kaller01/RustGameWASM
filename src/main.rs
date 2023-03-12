use crate::multiplayer::Event;
use controlls::{Controll, Controller};
use macroquad::prelude::*;
use macroquad_virtual_joystick::{Joystick, JoystickDirection};
use multiplayer::MultiplayerHandler;
use player::{Entity, Player};
use std::collections::HashMap;

pub mod controlls;
pub mod multiplayer;
pub mod player;
pub mod world;

#[macro_use]
extern crate lazy_static;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
use crate::wasm::WasmEventHandler;

use crate::world::World;

struct OtherPlayer {
    name: String,
    pos: Vec2,
    color: Color,
}

#[cfg(target_arch = "wasm32")]
fn get_multiplayer_handler() -> Box<dyn MultiplayerHandler> {
    Box::new(WasmEventHandler {})
}

#[cfg(not(target_arch = "wasm32"))]
fn get_multiplayer_handler() -> Box<dyn MultiplayerHandler> {
    Box::new(multiplayer::NotImplemented {})
}

#[macroquad::main("2D")]
async fn main() {
    let multiplayer_handler = get_multiplayer_handler();
    let mut other_players: HashMap<u32, OtherPlayer> = HashMap::new();
    let mut controller = Controller::default();

    

    // HashMap::new();

    // player_texture_down.

    // player_texture_down.set_filter(FilterMode::Nearest);
    // player_texture_left.set_filter(FilterMode::Nearest);
    // player_texture_right.set_filter(FilterMode::Nearest);
    // player_texture_up.set_filter(FilterMode::Nearest);

    let mut screen_size = (screen_width(), screen_height());
    let mut size = if screen_width() > screen_height() {
        screen_width()
    } else {
        screen_height()
    };
    //Changable settings (camera, player etc)
    let mut target = vec2(0., 0.);
    let mut z = 0.05;
    const MAX_ZOOM: f32 = 0.01;

    let mut world = World::generate();

    let mut player = Player::new(0., 0.).await;

    let mut player_joystick =
        Joystick::new(screen_width() * 0.8, screen_height() * 0.8, size * 0.1);
    let mut camera_joytstick =
        Joystick::new(screen_width() * 0.2, screen_height() * 0.8, size * 0.1);

    loop {
        clear_background(LIGHTGRAY);

        for event in multiplayer_handler.get_events() {
            match event {
                Event::PlayerUpdate {
                    name,
                    id,
                    x,
                    y,
                    r,
                    g,
                    b,
                } => match other_players.get_mut(&id) {
                    Some(player) => player.pos = vec2(x, y),
                    None => {
                        let new_player = OtherPlayer {
                            name,
                            pos: vec2(x, y),
                            color: Color::new(r, g, b, 1.),
                        };
                        other_players.insert(id, new_player);
                    }
                },
                Event::PlayerDisconnect { id } => {
                    other_players.remove(&id);
                }
                Event::YourColor { r, g, b } => player.set_color(Color::new(r, g, b, 1.)),
            }
        }

        multiplayer_handler.set_your_player_pos(player.get_position());

        // set_your_pos(player.get_position());

        if screen_size != (screen_width(), screen_height()) {
            screen_size = (screen_width(), screen_height());

            size = if screen_width() > screen_height() {
                screen_width()
            } else {
                screen_height()
            };

            player_joystick =
                Joystick::new(screen_width() * 0.8, screen_height() * 0.8, size * 0.1);
            camera_joytstick =
                Joystick::new(screen_width() * 0.2, screen_height() * 0.8, size * 0.1);
        }

        //Handle controlls
        {
            if controller.is(Controll::toggle_camera) {
                if controller.is(Controll::move_left) {
                    target.x -= 2.;
                }
                if controller.is(Controll::move_right) {
                    target.x += 2.;
                }
                if controller.is(Controll::move_down) {
                    target.y += 2.;
                }
                if controller.is(Controll::move_up) {
                    target.y -= 2.;
                }
            } else {
                let speed = 20.;
                let mut velocity = vec2(0., 0.);
                if controller.is(Controll::move_right) {
                    velocity.x = speed;
                }
                if controller.is(Controll::move_left) {
                    velocity.x = -speed;
                }
                if controller.is(Controll::move_up) {
                    velocity.y = -speed;
                }
                if controller.is(Controll::move_down) {
                    velocity.y = speed;
                }
                player.set_velocity(velocity);
                target = player.get_position();

                //Joystick
                let joystick_event = player_joystick.update();
                if joystick_event.direction != JoystickDirection::Idle {
                    player.set_velocity(
                        joystick_event.direction.to_local() * joystick_event.intensity * speed,
                    );
                }
            }
            if controller.is(Controll::zoom_in) {
                z *= 1.1;
            }
            if controller.is(Controll::zoom_out) {
                z *= 0.9;
            }

            if true {
                let joystick_event = camera_joytstick.update();
                match joystick_event.direction {
                    JoystickDirection::Up => z *= 1.01,
                    JoystickDirection::Down => z *= 0.99,
                    _ => (),
                }
            }
        }

        //Set camera for world
        let zoom = vec2(z, -z * (screen_width() / screen_height()));
        set_camera(&Camera2D {
            target: target,
            zoom: zoom,
            // viewport: Some((0,0,800,800)),
            // viewport: Some((0,(0.2*screen_height()).round() as i32,(screen_width()).round() as i32,(0.8*screen_height()).round() as i32)),
            ..Default::default()
        });

        //Calculate the area of which the camera can see
        let mut z = z;
        if z < MAX_ZOOM {
            z = MAX_ZOOM;
        }
        let zoom = vec2(z, z * (screen_width() / screen_height()));
        let size = 1. / zoom * 2.;
        let corner = target - size / 2.;
        let view = Rect::new(corner.x, corner.y, size.x, size.y);

        if (controller.is(Controll::toggle_generation)) {
            world.generate_at(view);
        }

        //render world within camera view
        world.render(view);
        //update player
        world.update_entity(&mut player, get_frame_time());
        //render player
        player.render();

        //Multiplayer tmp
        let (font_size, font_scale, font_aspect) = camera_font_scale(2.);
        let text_params = TextParams {
            font_size,
            font_scale,
            font_scale_aspect: font_aspect,
            color: BLACK,
            ..Default::default()
        };

        for other_player in other_players.values() {
            draw_text_ex(
                &other_player.name,
                other_player.pos.x + 1.5,
                other_player.pos.y + 0.5,
                text_params,
            );
            draw_circle(
                other_player.pos.x,
                other_player.pos.y,
                1.,
                other_player.color,
            );
        }

        if z <= MAX_ZOOM {
            draw_rectangle_lines(view.x, view.y, view.w, view.h, 5., PINK);
        }

        set_default_camera();

        draw_text(
            "ARROWS to move player (purple circle)",
            30.0,
            30.0,
            30.0,
            BLACK,
        );
        draw_text("Q-E to zoom camera", 30.0, 60.0, 30.0, BLACK);
        draw_text("WASD to move camera", 30.0, 90.0, 30.0, BLACK);
        draw_text("F to follow player", 30.0, 120.0, 30.0, BLACK);
        draw_text("G to toggle generation", 30.0, 150.0, 30.0, BLACK);
        draw_text("T to toggle touch controls", 30.0, 180.0, 30.0, BLACK);
        player_joystick.render();
        camera_joytstick.render();
        if z <= MAX_ZOOM {
            draw_text("Max zoom reached, pink is camera border, see the chunks load in and out as you move camera", 30.0, screen_height()*0.95, 30.0, BLACK);
        }

        next_frame().await
    }
}
