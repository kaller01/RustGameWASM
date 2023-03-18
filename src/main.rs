use crate::{controlls::ToggleControll, multiplayer::Event, player::load_textures};
use controlls::{Controll, Controller};
use macroquad::prelude::*;
use macroquad_virtual_joystick::{Joystick, JoystickDirection};
use multiplayer::MultiplayerHandler;
use player::{Entity, Player};
// use quad_url::*;
use std::collections::HashMap;
use touchbutton::Button;

pub mod controlls;
pub mod multiplayer;
pub mod player;
pub mod touchbutton;
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

const MAX_ZOOM: f32 = 0.005;
const MIN_ZOOM: f32 = 0.1;
const MAX_RENDER: f32 = 256.;

struct WorldCamera {
    mode: CameraMode,
    z: f32,
}

enum CameraMode {
    PlayerLock,
    Follow(Vec2),
}

struct TouchControll {
    attack: Button,
    roll: Button,
    joystick: Joystick,
    zoom_in: Button,
    zoom_out: Button,
}

impl TouchControll {
    fn update(&mut self) {
        self.attack.update();
        self.roll.update();
        self.zoom_in.update();
        self.zoom_out.update();
    }

    fn render(&self) {
        self.attack.render();
        self.roll.render();
        self.zoom_in.render();
        self.zoom_out.render();
        self.joystick.render();
    }
}

#[macroquad::main("2D")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);
    let mut multiplayer_handler = get_multiplayer_handler();
    let mut other_players: HashMap<u32, Player> = HashMap::new();
    let mut controller = Controller::default();

    //Changable settings (camera, player etc)
    let mut world_camera = WorldCamera {
        mode: CameraMode::PlayerLock,
        z: 0.04,
    };

    let mut screen_size = (screen_width(), screen_height());
    if screen_height() > screen_width() {
        controller.set(ToggleControll::Touch, true);
    }

    let mut world = World::generate();

    let (textures, texture_map) = load_textures().await;

    let mut player = Player::new_playable(-3., -10., &texture_map, &textures);

    let mut player2 = Player::new_other(
        String::from("Player2"),
        10.,
        10.,
        0.,
        0.,
        &texture_map,
        &textures,
    );
    let mut touch = make_touch_controlls();

    loop {
        clear_background(LIGHTGRAY);

        controller.update();
        touch.update();

        //Handle multiplayer
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
                        let new_player =
                            Player::new_other(name, x, y, vx, vy, &texture_map, &textures);
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
                    player.set_position(vec2(x, y));
                }
            }
        }

        multiplayer_handler.set_your_player_pos(player.get_position(), player.get_velocity());

        if screen_size != (screen_width(), screen_height()) {
            screen_size = (screen_width(), screen_height());
            touch = make_touch_controlls();
        }

        //Handle controlls
        handle_player_input(
            &mut player,
            &mut controller,
            &mut world_camera,
            &mut touch,
            &mut multiplayer_handler,
        );
        handle_debug_player(&mut controller, &mut player2, &mut multiplayer_handler);

        if !controller.is_enabled(ToggleControll::FreeZoom) {
            if world_camera.z < MAX_ZOOM {
                world_camera.z = MAX_ZOOM
            }
            if world_camera.z > MIN_ZOOM {
                world_camera.z = MIN_ZOOM
            }
        }

        //Controlls done, make view and render zones
        let z = world_camera.z;
        let target = match world_camera.mode {
            CameraMode::PlayerLock => {
                if controller.is_enabled(ToggleControll::Touch) {
                    let map_height = 1. / (z * (screen_width() / screen_height())) * 2.;
                    player.get_position() + vec2(0., map_height * 0.15)
                } else {
                    player.get_position()
                }
            }
            CameraMode::Follow(target) => target,
        };
        let view_zone = make_view_rect(target, make_view_size(z));
        let render_zone = make_view_rect(player.get_position(), make_view_size(z).normalize() * MAX_RENDER);
        let debug_render = controller.is_enabled(ToggleControll::DebugHitbox);

        //Update world
        {
            if controller.is_enabled(ToggleControll::TerrainGeneration) {
                if controller.is(Controll::ForceRender){
                    world.generate_at(view_zone, view_zone);
                } else {
                    world.generate_at(render_zone, view_zone);
                }
            }
            world.update_entity(&mut player, get_frame_time());
            for (_, other_player) in other_players.iter_mut() {
                if controller.is_enabled(ToggleControll::OtherAnimations) {
                    world.update_entity(other_player, get_frame_time());
                }
            }
        }

        //Render world
        {
            let zoom = vec2(z, -z * (screen_width() / screen_height()));
            set_camera(&Camera2D {
                target,
                zoom: zoom,
                ..Default::default()
            });
            let (font_size, font_scale, font_aspect) = camera_font_scale(2.);
            let text_params = TextParams {
                font_size,
                font_scale,
                font_scale_aspect: font_aspect,
                color: BLACK,
                ..Default::default()
            };

            world.render(view_zone);

            for (_, other_player) in other_players.iter_mut() {
                if controller.is_enabled(ToggleControll::OtherAnimations) {
                    world.update_entity(other_player, get_frame_time());
                }
                other_player.render(&text_params, debug_render);
            }

            player.render(&text_params, debug_render);

            if debug_render {
                draw_rectangle_lines(
                    render_zone.x,
                    render_zone.y,
                    render_zone.w,
                    render_zone.h,
                    1.,
                    PINK,
                )
            }
        }

        //Exprimental map
        if controller.is_enabled(ToggleControll::Map) {
            set_default_camera();
            draw_rectangle(
                screen_width() * 0.08,
                screen_height() * 0.08,
                screen_width() * 0.84,
                screen_height() * 0.84,
                BROWN,
            );

            let z = 0.008;
            let zoom = vec2(z, -z * (screen_width() / screen_height()));

            let target = player.get_position() / 8.;
            // let target = vec2(0.,0.);
            set_camera(&Camera2D {
                zoom,
                target,
                ..Default::default()
            });
            let map_zone = make_view_rect(target, make_view_size(z) * 0.8);
            world.generate_at(
                Rect::new(0., 0., 0., 0.),
                make_view_rect(target * 8., make_view_size(z) * 8.),
            );
            world.render_map(map_zone);
            let (font_size, font_scale, font_aspect) = camera_font_scale(2.);
            let text_params = TextParams {
                font_size,
                font_scale,
                font_scale_aspect: font_aspect,
                color: BLACK,
                ..Default::default()
            };
            for (_, other_player) in other_players.iter_mut() {
                if controller.is_enabled(ToggleControll::OtherAnimations) {
                    world.update_entity(other_player, get_frame_time());
                }
                let pos = other_player.get_position() / 8.;
                draw_circle(pos.x, pos.y, 0.5, RED);
                draw_text_ex(other_player.get_name(), pos.x, pos.y, text_params);
            }
            let pos = player.get_position() / 8.;
            draw_circle(pos.x, pos.y, 0.5, RED);
            draw_text_ex("You", pos.x, pos.y, text_params);

            //teleport
            if is_mouse_button_pressed(MouseButton::Middle) {
                let (mouse_x, mouse_y) = mouse_position();
                let pos = (vec2(mouse_x, mouse_y) - vec2(screen_width(), screen_height()) / 2.)
                    / vec2(screen_width(), screen_height())
                    * (make_view_size(z))
                    * 8.
                    + target * 8.;
                player.set_position(pos);
                controller.set(ToggleControll::Map, false);
            }
        }

        //render on screen
        {
            set_default_camera();
            draw_text("WASD to move player", 10.0, 30.0, 30.0, BLACK);
            draw_text("Q-E to zoom camera", 10.0, 60.0, 30.0, BLACK);
            draw_text("LeftShift to roll", 10.0, 90.0, 30.0, BLACK);
            draw_text("Space to swing sword", 10.0, 120.0, 30.0, BLACK);
            draw_text("M to open map", 10.0, 150.0, 30.0, BLACK);
            draw_text(
                &format!(
                    "{:.0}, {:.0}",
                    player.get_position().x,
                    player.get_position().y
                ),
                10.0,
                screen_height() - 10.,
                30.0,
                BLACK,
            );
            if controller.is_enabled(ToggleControll::Touch) {
                touch.render();
            }
        }

        next_frame().await
    }
}

fn make_touch_controlls() -> TouchControll {
    let size = if screen_width() > screen_height() {
        screen_width()
    } else {
        screen_height()
    };

    let player_joystick = Joystick::new(screen_width() * 0.7, screen_height() * 0.8, size * 0.15);
    // let mut camera_joytstick =
    //     Joystick::new(screen_width() * 0.2, screen_height() * 0.8, size * 0.1);
    let attack_button = Button::circle(
        vec2(screen_width() * 0.2, screen_height() * 0.75),
        size * 0.1,
    );
    let roll_button = Button::circle(
        vec2(screen_width() * 0.2, screen_height() * 0.85),
        size * 0.1,
    );
    let zoom_out_button = Button::rectangle(
        vec2(screen_width() * 0.4, screen_height() * 0.97),
        vec2(size * 0.1, size * 0.05),
    );

    let zoom_in_button = Button::rectangle(
        vec2(screen_width() * 0.6, screen_height() * 0.97),
        vec2(size * 0.1, size * 0.05),
    );
    TouchControll {
        attack: attack_button,
        roll: roll_button,
        joystick: player_joystick,
        zoom_in: zoom_in_button,
        zoom_out: zoom_out_button,
    }
}

fn make_view_size(z: f32) -> Vec2 {
    1. / (vec2(z, z * (screen_width() / screen_height()))) * 2.
}

fn make_view_rect(target: Vec2, size: Vec2) -> Rect {
    let corner = target - size / 2.;
    Rect::new(corner.x, corner.y, size.x, size.y)
}

fn handle_player_input(
    player: &mut Player,
    controller: &mut Controller,
    world_camera: &mut WorldCamera,
    touch: &mut TouchControll,
    multiplayer_handler: &mut Box<dyn MultiplayerHandler>,
) {
    match (
        &mut world_camera.mode,
        controller.is_enabled(ToggleControll::FreeCamera),
    ) {
        (CameraMode::PlayerLock, true) => {
            world_camera.mode = CameraMode::Follow(player.get_position())
        }
        (CameraMode::Follow(target), true) => {
            let size = make_view_size(world_camera.z) * 0.3;
            let zone = make_view_rect(*target, size);
            if !zone.contains(player.get_position()) {
                println!("Test: {}", (player.get_position() - *target) / size);
                *target += (player.get_position() - *target) / size;
            }
        }
        (CameraMode::Follow(_), false) => world_camera.mode = CameraMode::PlayerLock,
        _ => (),
    };

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

    //Joystick

    if controller.is(Controll::ZoomIn) {
        world_camera.z *= 1.1;
    }
    if controller.is(Controll::ZoomOut) {
        world_camera.z *= 0.9;
    }

    let mut action = None;

    if controller.is_enabled(ToggleControll::Touch) {
        let joystick_event = touch.joystick.update();
        if joystick_event.direction != JoystickDirection::Idle {
            player.set_velocity(
                joystick_event.direction.to_local().normalize() * joystick_event.intensity * speed,
            );
        }

        if touch.zoom_in.down() {
            world_camera.z *= 1.03;
        }
        if touch.zoom_out.down() {
            world_camera.z *= 0.97;
        }
        if touch.attack.down() {
            action = Some(player::BlockingAction::Attack)
        }
        if touch.roll.down() {
            action = Some(player::BlockingAction::Roll)
        }
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

fn handle_debug_player(
    controller: &mut Controller,
    player2: &mut Player,
    multiplayer_handler: &mut Box<dyn MultiplayerHandler>,
) {
    if controller.is_enabled(ToggleControll::SecondaryPlayer) {
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
        player2.set_position(player2.get_position() + player2.get_velocity() * get_frame_time());
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

// let mut my_boolean = true;

// egui_macroquad::ui(|ctx| {
//     egui::CentralPanel::default()
//         .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
//         .show(ctx, |ui| {
//             ui.add(egui::Slider::new(&mut z, 0.001..=0.1).text("My value"));
//             ui.add(egui::Label::new("Hello World!"));
//             ui.label("A shorter and more convenient way to add a label.");
//             if ui.button("Click me").clicked() {
//                 // take some action here
//             }
//             ui.checkbox(&mut my_boolean, "Checkbox");
//         });

//     // egui::Window::new("Tutorial").show(ctx, |ui| {
//     //     ui.label("Test");
//     //     if ui.button("Save").clicked() {
//     //         println!("Hola mi amigos")
//     //     }
//     // });
// });
// egui_macroquad::draw();
