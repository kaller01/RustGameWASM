use std::{collections::HashMap, hash::Hash};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

#[derive(Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Controll {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Roll,
    Block,
    ToggleTouch,
    //Camera
    ZoomIn,
    ZoomOut,
    ToggleCamera,
    //Dev
    ToggleMaxZoom,
    ToggleGeneration,
    ToggleOtherAnimation,
    //Player 2
    ToggleSecondaryPlayer,
    MoveSecondaryUp,
    MoveSecondaryDown,
    MoveSecondaryLeft,
    MoveSecondaryRight,
    SecondaryAttack,
    SecondaryRoll
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Setting {
    Toggle(bool),
}

#[derive(Eq, PartialEq, Clone)]
pub struct Controller {
    keymap: HashMap<Controll, KeyCode>,
    settings: HashMap<Controll, Setting>,
}

impl Controller {
    pub fn default() -> Controller {
        let mut keymap = HashMap::new();
        for controll in Controll::iter() {
            let keycode = match controll {
                Controll::MoveUp => KeyCode::W,
                Controll::MoveDown => KeyCode::S,
                Controll::MoveLeft => KeyCode::A,
                Controll::MoveRight => KeyCode::D,
                Controll::Attack => KeyCode::J,
                Controll::Roll => KeyCode::K,
                Controll::Block => KeyCode::L,
                Controll::ToggleTouch => KeyCode::T,
                Controll::ZoomIn => KeyCode::E,
                Controll::ZoomOut => KeyCode::Q,
                Controll::ToggleCamera => KeyCode::F,
                Controll::ToggleMaxZoom => KeyCode::I,
                Controll::ToggleGeneration => KeyCode::G,
                Controll::ToggleOtherAnimation => KeyCode::O,
                Controll::ToggleSecondaryPlayer => KeyCode::Kp5,
                Controll::MoveSecondaryUp => KeyCode::Kp8,
                Controll::MoveSecondaryDown => KeyCode::Kp2,
                Controll::MoveSecondaryLeft => KeyCode::Kp4,
                Controll::MoveSecondaryRight => KeyCode::Kp6,
                Controll::SecondaryAttack => KeyCode::Kp0,
                Controll::SecondaryRoll => KeyCode::KpMultiply
            };
            keymap.insert(controll, keycode);
        }
        Controller {
            keymap,
            settings: HashMap::from([
                (Controll::ToggleMaxZoom, Setting::Toggle(false)),
                (Controll::ToggleCamera, Setting::Toggle(false)),
                (Controll::ToggleGeneration, Setting::Toggle(true)),
                (Controll::ToggleSecondaryPlayer, Setting::Toggle(false)),
                (Controll::ToggleOtherAnimation, Setting::Toggle(true)),
                (Controll::ToggleTouch, Setting::Toggle(true)),
            ]),
        }
    }

    fn get_key(&self, controll: &Controll) -> KeyCode {
        match self.keymap.get(&controll) {
            Some(key) => {
                return key.to_owned();
            }
            None => todo!(),
        }
    }

    pub fn is(&mut self, controll: Controll) -> bool {
        match controll {
            Controll::MoveUp
            | Controll::MoveDown
            | Controll::MoveLeft
            | Controll::MoveRight
            | Controll::Attack
            | Controll::Roll
            | Controll::Block
            | Controll::ZoomIn
            | Controll::ZoomOut
            | Controll::MoveSecondaryUp
            | Controll::MoveSecondaryDown
            | Controll::MoveSecondaryLeft
            | Controll::MoveSecondaryRight
            | Controll::SecondaryAttack
            | Controll::SecondaryRoll => is_key_down(self.get_key(&controll)),
            Controll::ToggleCamera
            | Controll::ToggleMaxZoom
            | Controll::ToggleGeneration
            | Controll::ToggleSecondaryPlayer
            | Controll::ToggleOtherAnimation
            | Controll::ToggleTouch => {
                let key_pressed = is_key_pressed(self.get_key(&controll));
                match self.settings.get(&controll) {
                    Some(setting) => {
                        match setting {
                            Setting::Toggle(enabled) => {
                                if key_pressed {
                                    let tmp = enabled.to_owned();
                                    self.settings.insert(
                                        controll.to_owned(),
                                        Setting::Toggle(!enabled.to_owned()),
                                    );
                                    return tmp;
                                } else {
                                    return *enabled;
                                }
                            }
                        };
                    }
                    None => todo!(),
                }
            }
        }
    }
}
