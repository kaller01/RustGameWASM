use crate::{multiplayer::Event, player::load_textures, controlls::ToggleControll};
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

#[cfg(target_arch = "wasm32")]
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
    Box::new(multiplayer::DevLocalMultiplayer::new())
}

#[macroquad::main("2D")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);
    let mut multiplayer_handler = get_multiplayer_handler();
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

    let (textures, texture_map) = load_textures().await;

    let mut player = Player::new_playable(-3., -10., &texture_map, &textures);

    let mut player2 = Player::new_other(String::from("Player2"), 10., 10., 0., 0., &texture_map, &textures);

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
                    vy,
                } => match other_players.get_mut(&id) {
                    Some(other_player) => {
                        other_player.set_position(vec2(x, y));
                        other_player.set_velocity(vec2(vx, vy))
                    }
                    None => {
                        let new_player = Player::new_other(name, x, y, vx, vy, &texture_map, &textures);
                        other_players.insert(id, new_player);
                    }
                },
                Event::PlayerDisconnect { id } => {
                    other_players.remove(&id);
                }
                Event::PlayerAction {
                    id,
                    x,
                    y,
                    direction,
                    action,
                } => match other_players.get_mut(&id) {
                    Some(other_player) => {
                        other_player.force_action(action, vec2(x, y), direction);
                        match action {
                            player::BlockingAction::Attack => {
                                debug!("Distance {}", player.get_position().distance(vec2(x, y)));
                                if player.get_position().distance(vec2(x, y)) < 5. {
                                    player.kill();
                                    multiplayer_handler.upstream_event(Event::PlayerAction {
                                        id: 0,
                                        x: player.get_position().x,
                                        y: player.get_position().y,
                                        direction: player.get_direction(),
                                        action: player::BlockingAction::Dying,
                                    })
                                }
                            }
                            _ => (),
                        }
                    }
                    None => (),
                },
                Event::CommandTeleport { x, y } => {
                    player.set_position(vec2(x,y));
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
                let speed = 15.;
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

            let mut action = None;

            let joystick_event = camera_joytstick.update();
            match joystick_event.direction {
                JoystickDirection::Up => z *= 1.01,
                JoystickDirection::Down => z *= 0.99,
                JoystickDirection::Left => action = Some(player::BlockingAction::Attack),
                JoystickDirection::Right => action = Some(player::BlockingAction::Roll),
                _ => (),
            }

            if controller.is(Controll::Attack) {
                action = Some(player::BlockingAction::Attack)
            }
            if controller.is(Controll::Roll) {
                action = Some(player::BlockingAction::Roll)
            }
            if controller.is(Controll::Block) {
                action = Some(player::BlockingAction::Block)
            }

            match action {
                Some(action) => match player.try_action(action) {
                    Ok(_) => multiplayer_handler.upstream_event(Event::PlayerAction {
                        id: 0,
                        x: player.get_position().x,
                        y: player.get_position().y,
                        direction: player.get_direction(),
                        action: action,
                    }),
                    Err(_) => (),
                },
                None => (),
            }
        }

        {
            //Player 2 controlls
            if controller.enabled(ToggleControll::SecondaryPlayer) {
                player2.update(&world::TileInteraction::Walkable, get_frame_time());
                let speed = 20.;
                let mut velocity = vec2(0., 0.);
                if controller.is(Controll::MoveSecondaryRight) {
                    velocity.x = speed;
                }
                if controller.is(Controll::MoveSecondaryLeft) {
                    velocity.x = -speed;
                }
                if controller.is(Controll::MoveSecondaryUp) {
                    velocity.y = -speed;
                }
                if controller.is(Controll::MoveSecondaryDown) {
                    velocity.y = speed;
                }
                velocity = velocity.normalize_or_zero() * speed;
                player2.set_velocity(velocity);
                player2.set_position(
                    player2.get_position() + player2.get_velocity() * get_frame_time(),
                );
                let player_update = Event::PlayerUpdate {
                    name: String::from("Player2"),
                    id: 0,
                    x: player2.get_position().x,
                    y: player2.get_position().y,
                    vx: player2.get_velocity().x,
                    vy: player2.get_velocity().y,
                };
                multiplayer_handler.downstream_event(player_update);
                let mut action = None;
                if controller.is(Controll::SecondaryAttack) {
                    action = Some(player::BlockingAction::Attack)
                }
                if controller.is(Controll::SecondaryRoll) {
                    action = Some(player::BlockingAction::Roll)
                }

                match action {
                    Some(action) => match player2.try_action(action) {
                        Ok(_) => {
                            let event = Event::PlayerAction {
                                id: 0,
                                x: player2.get_position().x,
                                y: player2.get_position().y,
                                direction: player2.get_direction(),
                                action,
                            };
                            multiplayer_handler.downstream_event(event)
                        }
                        Err(_) => (),
                    },
                    None => (),
                }
            }
        }

        //Set camera for world

        if !controller.enabled(ToggleControll::FreeZoom) && z < MAX_ZOOM {
            z = MAX_ZOOM
        }

        let zoom = vec2(z, -z * (screen_width() / screen_height()));

        set_camera(&Camera2D {
            target: target,
            zoom: zoom,
            // viewport: Some((0,0,800,800)),
            // viewport: Some((0,(0.2*screen_height()).round() as i32,(screen_width()).round() as i32,(0.8*screen_height()).round() as i32)),
            ..Default::default()
        });

        //Calculate the area of which the camera can see
        let zoom = vec2(z, z * (screen_width() / screen_height()));
        let size = 1. / zoom * 2.;
        let corner = target - size / 2.;
        let view = Rect::new(corner.x, corner.y, size.x, size.y);

        if controller.enabled(ToggleControll::TerrainGeneration) {
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

        let debug_render = controller.enabled(ToggleControll::DebugHitbox);

        //render world within camera view
        world.render(view);
        //update player
        world.update_entity(&mut player, get_frame_time());

        for (_, other_player) in other_players.iter_mut() {
            if controller.enabled(ToggleControll::OtherAnimations) {
                world.update_entity(other_player, get_frame_time());
            }
            other_player.render(&text_params, debug_render);
        }

        player.render(&text_params,debug_render);

        // if z <= MAX_ZOOM {
        //     if(controller.is(Controll::ToggleDev)){
        //         draw_rectangle_lines(view.x, view.y, view.w, view.h, 5., PINK);
        //     } else {
        //         // z = MAX_ZOOM;
        //     }
        // }

        set_default_camera();

        draw_text("WASD to move player", 10.0, 30.0, 30.0, BLACK);
        draw_text("Q-E to zoom camera", 10.0, 60.0, 30.0, BLACK);
        draw_text(
            &format!("{}, {}", player.get_position().x, player.get_position().y),
            10.0,
            screen_height() - 10.,
            30.0,
            BLACK,
        );
        // draw_text("WASD to move camera", 30.0, 90.0, 30.0, BLACK);
        // draw_text("F to follow player", 30.0, 120.0, 30.0, BLACK);
        // draw_text("G to toggle generation", 30.0, 150.0, 30.0, BLACK);
        // draw_text("T to toggle touch controls", 30.0, 180.0, 30.0, BLACK);
        if controller.enabled(ToggleControll::Touch) {
            player_joystick.render();
            camera_joytstick.render();
        }

        next_frame().await
    }
}
