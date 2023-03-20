use std::{collections::HashMap, hash::Hash};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

trait KeyMapped {
    fn to_key(&self) -> KeyCode;
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum Controll {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Roll,
    Block,
    //Camera
    ZoomIn,
    ZoomOut,
    ForceRender,
    //Player 2
    MoveSecondaryUp,
    MoveSecondaryDown,
    MoveSecondaryLeft,
    MoveSecondaryRight,
    SecondaryAttack,
    SecondaryRoll,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, EnumIter)]
pub enum ToggleControll {
    Touch,
    FreeCamera,
    FreeZoom,
    TerrainGeneration,
    OtherAnimations,
    SecondaryPlayer,
    DebugHitbox,
    Map
}

impl KeyMapped for Controll {
    fn to_key(&self) -> KeyCode {
        match self {
            Controll::MoveUp => KeyCode::W,
            Controll::MoveDown => KeyCode::S,
            Controll::MoveLeft => KeyCode::A,
            Controll::MoveRight => KeyCode::D,
            Controll::Attack => KeyCode::J,
            Controll::Roll => KeyCode::Space,
            Controll::Block => KeyCode::L,
            Controll::ZoomIn => KeyCode::E,
            Controll::ZoomOut => KeyCode::Q,
            Controll::ForceRender => KeyCode::R,
            Controll::MoveSecondaryUp => KeyCode::Kp8,
            Controll::MoveSecondaryDown => KeyCode::Kp2,
            Controll::MoveSecondaryLeft => KeyCode::Kp4,
            Controll::MoveSecondaryRight => KeyCode::Kp6,
            Controll::SecondaryAttack => KeyCode::Kp0,
            Controll::SecondaryRoll => KeyCode::KpMultiply,
        }
    }
}

impl KeyMapped for ToggleControll {
    fn to_key(&self) -> KeyCode {
        match self {
            ToggleControll::Touch => KeyCode::T,
            ToggleControll::FreeCamera => KeyCode::F,
            ToggleControll::FreeZoom => KeyCode::I,
            ToggleControll::TerrainGeneration => KeyCode::G,
            ToggleControll::OtherAnimations => KeyCode::O,
            ToggleControll::SecondaryPlayer => KeyCode::Kp5,
            ToggleControll::DebugHitbox => KeyCode::H,
            ToggleControll::Map => KeyCode::M,
        }
    }
}

impl ToggleControll {
    fn default(&self) -> bool {
        match self {
            ToggleControll::Touch => false,
            ToggleControll::FreeCamera => false,
            ToggleControll::FreeZoom => false,
            ToggleControll::TerrainGeneration => true,
            ToggleControll::OtherAnimations => true,
            ToggleControll::SecondaryPlayer => false,
            ToggleControll::DebugHitbox => false,
            ToggleControll::Map => false,
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Controller {
    toggles: HashMap<ToggleControll, bool>,
}

impl Controller {
    pub fn default() -> Controller {
        let mut toggles = HashMap::new();
        for toggle_controll in ToggleControll::iter() {
            toggles.insert(toggle_controll, toggle_controll.default());
        }

        Controller { toggles }
    }

    pub fn update(&mut self) {
        for toggle in ToggleControll::iter() {
            let key_pressed = is_key_pressed(toggle.to_key());
            let enabled = self.toggles.get(&toggle).unwrap();
            if key_pressed {
                self.toggles.insert(toggle, !enabled.to_owned());
            }
        }
    }

    pub fn set(&mut self, toggle: ToggleControll, enabled: bool){
        self.toggles.insert(toggle, enabled);
    }

    pub fn is_enabled(&self, toggle: ToggleControll) -> bool {
        return *self.toggles.get(&toggle).unwrap();
    }

    pub fn is(&mut self, controll: Controll) -> bool {
        is_key_down(controll.to_key())
    }
}
