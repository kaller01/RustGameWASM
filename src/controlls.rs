use std::{collections::HashMap, hash::Hash};

use macroquad::prelude::{KeyCode, is_key_pressed, is_key_down};

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum Controll {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    SpinLeft,
    SpinRight,
    //Camera
    ZoomIn,
    ZoomOut,
    ToggleCamera,
    //Dev
    ToggleDev,
    ToggleGeneration,
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
                (Controll::MoveUp, KeyCode::W),
                (Controll::MoveLeft, KeyCode::A),
                (Controll::MoveRight, KeyCode::D),
                (Controll::MoveDown, KeyCode::S),
                (Controll::SpinLeft, KeyCode::Left),
                (Controll::SpinRight, KeyCode::Right),
                (Controll::ZoomIn, KeyCode::E),
                (Controll::ZoomOut, KeyCode::Q),
                (Controll::ToggleDev, KeyCode::I),
                (Controll::ToggleCamera, KeyCode::F),
                (Controll::ToggleGeneration, KeyCode::G),
            ]),
            settings: HashMap::from([
                (Controll::ToggleDev, Setting::toggle(false)),
                (Controll::ToggleCamera, Setting::toggle(false)),
                (Controll::ToggleGeneration, Setting::toggle(true)),
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
            Controll::MoveUp
            | Controll::MoveDown
            | Controll::MoveLeft
            | Controll::MoveRight
            | Controll::SpinLeft
            | Controll::SpinRight
            | Controll::ZoomIn
            | Controll::ZoomOut => 
                is_key_down(self.get_key(&controll)),
            | Controll::ToggleCamera
            | Controll::ToggleDev
            | Controll::ToggleGeneration => {
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
