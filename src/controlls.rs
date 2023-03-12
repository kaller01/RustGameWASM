use std::{collections::HashMap, hash::Hash};

use macroquad::prelude::{KeyCode, is_key_pressed, is_key_down};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Controll {
    move_up,
    move_down,
    move_left,
    move_right,
    spin_left,
    spin_right,
    //Camera
    zoom_in,
    zoom_out,
    toggle_camera,
    //Dev
    toggle_dev,
    toggle_generation,
}

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Setting {
    toggle(bool)
}

#[derive(Eq, PartialEq, Clone)]
pub struct Controller {
    keymap: HashMap<Controll, KeyCode>,
    settings: HashMap<Controll, Setting>,
}

impl Controller {
    pub fn default() -> Controller {
        Controller {
            keymap: HashMap::from([
                (Controll::move_up, KeyCode::W),
                (Controll::move_left, KeyCode::A),
                (Controll::move_right, KeyCode::D),
                (Controll::move_down, KeyCode::S),
                (Controll::spin_left, KeyCode::Left),
                (Controll::spin_right, KeyCode::Right),
                (Controll::zoom_in, KeyCode::E),
                (Controll::zoom_out, KeyCode::Q),
                (Controll::toggle_dev, KeyCode::I),
                (Controll::toggle_camera, KeyCode::F),
                (Controll::toggle_generation, KeyCode::G),
            ]),
            settings: HashMap::from([
                (Controll::toggle_dev, Setting::toggle(false)),
                (Controll::toggle_camera, Setting::toggle(false)),
                (Controll::toggle_generation, Setting::toggle(true)),
            ]),
        }
    }

    fn get_key(&self, controll: &Controll) -> KeyCode {
        match self.keymap.get(&controll) {
            Some(key) => {
                return key.to_owned();
            },
            None => todo!(),
        }
    }

    pub fn is(&mut self, controll: Controll) -> bool {
        match controll {
            Controll::move_up
            | Controll::move_down
            | Controll::move_left
            | Controll::move_right
            | Controll::spin_left
            | Controll::spin_right
            | Controll::zoom_in
            | Controll::zoom_out => 
                is_key_down(self.get_key(&controll)),
            | Controll::toggle_camera
            | Controll::toggle_dev
            | Controll::toggle_generation => {
                let key_pressed =  is_key_pressed(self.get_key(&controll));
                match self.settings.get(&controll) {
                    Some(setting) => {
                        match setting {
                            Setting::toggle(enabled) => {
                                if key_pressed {
                                    let tmp = enabled.to_owned();
                                    self.settings.insert(controll.to_owned(), Setting::toggle(!enabled.to_owned()));
                                    return tmp;
                                } else {
                                    return *enabled;
                                }
                            },
                        };
                    },
                    None => todo!(),
                }
            },
        }
    }
}

pub struct KeyMapping {
    //Movement
    move_up: KeyCode,
    move_down: KeyCode,
    move_left: KeyCode,
    move_right: KeyCode,
    toggle_touch: KeyCode,
    spin_left: KeyCode,
    spin_right: KeyCode,
    //Camera
    zoom_in: KeyCode,
    zoom_out: KeyCode,
    detach_camera: KeyCode,
    //Dev
    toggle_dev: KeyCode,
    toggle_generation: KeyCode,
}

impl KeyMapping {
    pub fn default() -> KeyMapping {
        KeyMapping {
            //Movement
            move_up: KeyCode::W,
            move_down: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::R,
            toggle_touch: KeyCode::T,
            spin_left: KeyCode::Left,
            spin_right: KeyCode::Right,
            //Camera
            zoom_in: KeyCode::E,
            zoom_out: KeyCode::Q,
            detach_camera: KeyCode::F,
            //Dev
            toggle_dev: KeyCode::I,
            toggle_generation: KeyCode::G,
        }
    }
}
