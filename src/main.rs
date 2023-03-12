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
    let mut other_players: HashMap<u32, Player> = HashMap::new();
    let mut controller = Controller::default();

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

    let textures = load_textures().await;

    let mut player = Player::new_playable(0., 0., &textures);

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
                    vx,
                    vy
                } => match other_players.get_mut(&id) {
                    Some(player) => {
                        player.set_position(vec2(x, y));
                        player.set_velocity(vec2(vx,vy))

                    },
                    None => {
                        // let new_player = OtherPlayer {
                        //     name,
                        //     pos: vec2(x, y),
                        //     color: Color::new(r, g, b, 1.),
                        // };
                        let new_player = Player::new_other(name, x, y, vx, vy, &textures);
                        other_players.insert(id, new_player);
                    }
                },
                Event::PlayerDisconnect { id } => {
                    other_players.remove(&id);
                }
            }
        }

        multiplayer_handler.set_your_player_pos(player.get_position(), player.get_velocity());

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
            if controller.is(Controll::ToggleCamera) {
                if controller.is(Controll::MoveLeft) {
                    target.x -= 2.;
                }
                if controller.is(Controll::MoveRight) {
                    target.x += 2.;
                }
                if controller.is(Controll::MoveDown) {
                    target.y += 2.;
                }
                if controller.is(Controll::MoveUp) {
                    target.y -= 2.;
                }
            } else {
                let speed = 20.;
                let mut velocity = vec2(0., 0.);
                if controller.is(Controll::MoveRight) {
                    velocity.x = speed;
                }
                if controller.is(Controll::MoveLeft) {
                    velocity.x = -speed;
                }
                if controller.is(Controll::MoveUp) {
                    velocity.y = -speed;
                }
                if controller.is(Controll::MoveDown) {
                    velocity.y = speed;
                }
                velocity = velocity.normalize_or_zero() * speed;
                player.set_velocity(velocity);
                target = player.get_position();

                //Joystick
                let joystick_event = player_joystick.update();
                if joystick_event.direction != JoystickDirection::Idle {
                    player.set_velocity(
                        joystick_event.direction.to_local().normalize()
                            * joystick_event.intensity
                            * speed,
                    );
                }
            }
            if controller.is(Controll::ZoomIn) {
                z *= 1.1;
            }
            if controller.is(Controll::ZoomOut) {
                z *= 0.9;
            }

            let joystick_event = camera_joytstick.update();
            match joystick_event.direction {
                JoystickDirection::Up => z *= 1.01,
                JoystickDirection::Down => z *= 0.99,
                _ => (),
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

        if (controller.is(Controll::ToggleGeneration)) {
            world.generate_at(view);
        }

        let (font_size, font_scale, font_aspect) = camera_font_scale(2.);
        let text_params = TextParams {
            font_size,
            font_scale,
            font_scale_aspect: font_aspect,
            color: BLACK,
            ..Default::default()
        };

        //render world within camera view
        world.render(view);
        //update player
        world.update_entity(&mut player, get_frame_time());
        

        for (_, other_player) in other_players.iter_mut() {
            world.update_entity(other_player, get_frame_time());
            other_player.render(&text_params);
        }

        player.render(&text_params);

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

async fn load_textures() -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    let mut textures_names: Vec<String> = Vec::new();

    for action in ["walk", "swim", "idle"] {
        for direction in ["down", "left", "right", "up"] {
            for step in ["1", "2", "3", "4"] {
                textures_names.push(format!("{} {}{}", action, direction, step).to_owned())
            }
        }
    }

    for texture_name in textures_names {
        let texture = load_texture(&format!("textures/{}.png", texture_name))
            .await
            .unwrap();
        texture.set_filter(FilterMode::Nearest);
        textures.insert(texture_name.to_owned(), texture);
    }
    textures
}
