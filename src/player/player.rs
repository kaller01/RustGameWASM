use std::collections::HashMap;

use macroquad::prelude::*;

use crate::world::{entity::*, tile::*};

use super::{Player, animation::*, ATTACK_COOLDOWN, BlockingAction, Interaction};

impl BlockingAction {
    pub fn to_interaction(&self) -> Interaction {
        match self {
            BlockingAction::Attack => Interaction::Attack,
            BlockingAction::Roll => Interaction::Roll,
            BlockingAction::Block => Interaction::Block,
            BlockingAction::Dying => Interaction::Dying,
        }
    }
}

impl Direction {
    pub fn from_vec2(vec: Vec2) -> Option<Direction> {
        if vec.x > 0. && vec.y == 0. {
            Some(Direction::Right)
        } else if vec.x < 0. && vec.y == 0. {
            Some(Direction::Left)
        } else if vec.x == 0. && vec.y > 0. {
            Some(Direction::Down)
        } else if vec.x == 0. && vec.y < 0. {
            Some(Direction::Up)
        } else if vec.x > 0. {
            Some(Direction::Right)
        } else if vec.x < 0. {
            Some(Direction::Left)
        } else {
            None
        }
    }
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => vec2(0., -1.),
            Direction::Down => vec2(0., 1.),
            Direction::Left => vec2(-1., 0.),
            Direction::Right => vec2(1., 0.),
        }
    }
}

impl Player<'_> {
    pub fn new_playable<'a>(
        x: f32,
        y: f32,
        animations: &'a HashMap<(Interaction, Direction), Animation>,
        textures: &'a Texture2D,
    ) -> Player<'a> {
        Player {
            name: "You".to_owned(),
            pos: vec2(x, y),
            v: vec2(0., 0.),
            animations,
            textures,
            keyframe: KeyFrame::Free(0, Interaction::Idle),
            keyframe_timer: 0.,
            direction: Direction::Right,
            cooldowns: HashMap::new(),
            world_events: Vec::new(),
            local_player: true,
        }
    }

    pub fn new_other<'a>(
        name: String,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        animations: &'a HashMap<(Interaction, Direction), Animation>,
        textures: &'a Texture2D,
    ) -> Player<'a> {
        Player {
            name,
            pos: vec2(x, y),
            v: vec2(vx, vy),
            animations,
            textures,
            keyframe: KeyFrame::Free(0, Interaction::Idle),
            keyframe_timer: 0.,
            direction: Direction::Right,
            cooldowns: HashMap::new(),
            world_events: Vec::new(),
            local_player: false
        }
    }

    pub fn set_action(&mut self, action: BlockingAction) {
        let velocity = match action {
            BlockingAction::Roll => self.direction.to_vec2() * 30.,
            _ => vec2(0., 0.),
        };

        self.set_velocity(velocity);
        self.keyframe = KeyFrame::Blocking(0, action);

        match action {
            BlockingAction::Attack => self
                .world_events
                .push(EntityWorldEvent::Destroy(self.get_direction())),
            _ => (),
        }
    }

    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    pub fn kill(&mut self) {
        self.set_action(BlockingAction::Dying)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn respawn(&mut self) {
        let size = 50;
        let (x, y) = (rand::gen_range(-size, size), rand::gen_range(-size, size));
        self.pos += vec2(x as f32, y as f32);
    }

    //used for multiplayer
    pub fn force_action(&mut self, action: BlockingAction, pos: Vec2, direction: Direction) {
        self.pos = pos;
        self.direction = direction;
        self.set_action(action);
    }

    pub fn try_action(&mut self, action: BlockingAction) -> Result<(), ()> {
        match self.keyframe {
            KeyFrame::Blocking(_, _) => Err(()),
            KeyFrame::Free(_, interaction) => match interaction {
                Interaction::Walk | Interaction::Idle => match self.cooldowns.get(&action) {
                    Some(_) => Err(()),
                    None => {
                        self.set_action(action);
                        Ok(())
                    }
                },
                _ => Err(()),
            },
        }
    }

    fn advance_keyframe(&mut self) {
        match self.keyframe {
            KeyFrame::Blocking(frame, action) => {
                self.keyframe = KeyFrame::Blocking(frame + 1, action)
            }
            KeyFrame::Free(frame, interaction) => {
                self.keyframe = KeyFrame::Free(frame + 1, interaction)
            }
        }
    }

    fn current_keyframe(&mut self) -> u8 {
        match self.keyframe {
            KeyFrame::Blocking(frame, _) | KeyFrame::Free(frame, _) => frame,
        }
    }

    //Called when animation is changing
    fn free_keyframe(&mut self, next_interaction: Interaction) {
        match self.keyframe {
            KeyFrame::Blocking(_, action) => match action {
                BlockingAction::Attack => {
                    self.cooldowns.insert(action, ATTACK_COOLDOWN);
                }
                BlockingAction::Roll => {
                    self.cooldowns.insert(action, 1.);
                }
                BlockingAction::Dying => {
                    if self.local_player {
                        self.respawn()
                    }                    
                },
                _ => (),
            },
            KeyFrame::Free(_, _) => {}
        }
        self.keyframe = KeyFrame::Free(0, next_interaction);
    }

    //called when cooldown is reset.
    fn reset_cooldown(&mut self, from_action: BlockingAction) {
        match from_action {
            BlockingAction::Attack => (),
            BlockingAction::Roll => (),
            BlockingAction::Block => (),
            BlockingAction::Dying => self.set_position(vec2(0., 0.)),
        }
        self.cooldowns.remove(&from_action);
    }

    pub fn render(&mut self, text_params: &TextParams, debug: bool) {
        let interaction = match self.keyframe {
            KeyFrame::Blocking(_, action) => action.to_interaction(),
            KeyFrame::Free(_, interaction) => interaction,
        };

        let x = self.pos.x;
        let y = self.pos.y;

       

        match self.animations.get(&(interaction, self.direction)) {
            Some(texture_data) => {
                let pixel_pos = texture_data
                    .texture_pos
                    .get(self.current_keyframe() as usize)
                    .unwrap();

                draw_texture_ex(
                    *self.textures,
                    self.pos.x - 8.,
                    self.pos.y - 8.,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(16., 16.)),
                        source: Some(Rect::new(
                            pixel_pos.x,
                            pixel_pos.y,
                            pixel_pos.w,
                            pixel_pos.h,
                        )),
                        ..Default::default()
                    },
                );
            }
            None => {
                draw_circle(self.pos.x, self.pos.y, 1., RED);
            }
        }

        match self.cooldowns.get(&BlockingAction::Attack) {
            Some(time) => {
                let cooldown = (ATTACK_COOLDOWN-time)/ATTACK_COOLDOWN;
                draw_rectangle(x-2., y+2., 4.*cooldown, 0.5, color_u8!(255,124,0,200));
            },
            None => (),
        };
        if debug {
            draw_circle(self.pos.x, self.pos.y, 0.1, RED);
            draw_circle_lines(self.pos.x, self.pos.y, 2.5, 0.1, RED);
        }

        draw_text_ex(&self.name, self.pos.x + 1., self.pos.y - 2., *text_params);
    }
    pub fn set_velocity(&mut self, velocity: Vec2) {
        match self.keyframe {
            KeyFrame::Blocking(_, _) => (),
            KeyFrame::Free(_, _) => self.v = velocity,
        }
    }
}



impl WorldEntity for Player<'_> {
    fn get_velocity(&self) -> Vec2 {
        self.v
    }

    fn get_position(&self) -> Vec2 {
        self.pos
    }

    fn set_position(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn update(&mut self, tile_interaction: &TileInteraction, tile_action: &TileAction, time: f32) {
        let next_possible_interaction = match tile_interaction {
            TileInteraction::Block => Interaction::Idle,
            TileInteraction::Walkable => {
                if self.v.length() == 0. {
                    Interaction::Idle
                } else {
                    Interaction::Walk
                }
            }
            TileInteraction::Swimmable => Interaction::Swim,
            TileInteraction::Crawl => Interaction::Walk,
        };

        match (self.keyframe, tile_action) {
            (KeyFrame::Free(_, _), TileAction::Death) => self.kill(),
            _ => (),
        }

        let interaction = match self.keyframe {
            KeyFrame::Blocking(_, action) => action.to_interaction(),
            KeyFrame::Free(_, interaction) => interaction,
        };

        {
            let time = match (interaction, tile_interaction) {
                (Interaction::Walk, TileInteraction::Crawl) => time * 0.4,
                _ => time,
            };

            //Keyframe timer
            match self.animations.get(&(interaction, self.direction)) {
                Some(texture_data) => {
                    let time_step = texture_data.time_step;
                    let steps = texture_data.texture_pos.len();
                    if self.keyframe_timer > time_step {
                        self.advance_keyframe();
                        if self.current_keyframe() == steps as u8 {
                            match self.keyframe {
                                //Keyframe loop complete
                                KeyFrame::Blocking(_, _) => {
                                    self.free_keyframe(next_possible_interaction)
                                }
                                KeyFrame::Free(_, interaction) => {
                                    if interaction != next_possible_interaction {
                                        self.free_keyframe(next_possible_interaction);
                                    } else {
                                        //Keyframe looping
                                        self.keyframe = KeyFrame::Free(0, interaction);
                                    }
                                }
                            }
                        } else {
                            //If keyframe is free, change animation mid keyframe
                            match self.keyframe {
                                KeyFrame::Blocking(_, _) => (),
                                KeyFrame::Free(_, interaction) => {
                                    if interaction != next_possible_interaction {
                                        self.free_keyframe(next_possible_interaction)
                                    }
                                }
                            }
                        }
                        self.keyframe_timer = 0.;
                    }
                    self.keyframe_timer += time;
                }
                None => {}
            }
        }

        //Direction is allowed to be changed if the keyframe is free
        match self.keyframe {
            KeyFrame::Blocking(_, _) => (),
            KeyFrame::Free(_, _) => match Direction::from_vec2(self.v) {
                None => {}
                Some(direction) => self.direction = direction,
            },
        }

        //Cooldown timer
        let mut to_be_removed = Vec::new();
        for (action, cooldown) in self.cooldowns.iter_mut() {
            let remaining_cooldown = *cooldown - time;
            if remaining_cooldown < 0. {
                to_be_removed.push(action.clone());
            } else {
                *cooldown = remaining_cooldown;
            }
        }
        for to_be_removed in to_be_removed {
            self.reset_cooldown(to_be_removed);
        }
    }

    fn get_world_event(&mut self) -> EntityWorldEvent {
        match self.world_events.pop() {
            Some(event) => event,
            None => EntityWorldEvent::None,
        }
    }
}