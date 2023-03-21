use macroquad::prelude::*;

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub struct Tile {
    pub texture: TileTexture,
    pub interaction: TileInteraction,
    pub action: TileAction,
}

impl Tile {
    pub fn generate(n: f64) -> Tile {
        let (texture, interaction, action) = if n < 0.25 {
            (
                TileTexture::DeepWater,
                TileInteraction::Walkable,
                TileAction::Death,
            )
        } else if n < 0.43 {
            (
                TileTexture::Water,
                TileInteraction::Swimmable,
                TileAction::None,
            )
        } else if n < 0.5 {
            (
                TileTexture::ShallowWater,
                TileInteraction::Crawl,
                TileAction::None,
            )
        } else if n < 0.52 {
            (
                TileTexture::Sand,
                TileInteraction::Walkable,
                TileAction::None,
            )
        } else if n < 0.7 {
            (
                TileTexture::Grass,
                TileInteraction::Walkable,
                TileAction::None,
            )
        } else if n < 0.72 {
            (
                TileTexture::Dirt,
                TileInteraction::Block,
                TileAction::Destroyable,
            )
        } else if n < 0.85 {
            (
                TileTexture::Stone,
                TileInteraction::Block,
                TileAction::Death,
            )
        } else {
            (
                TileTexture::SnowyMountain,
                TileInteraction::Block,
                TileAction::Death,
            )
        };
        Tile {
            texture,
            interaction,
            action,
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            texture: TileTexture::Grass,
            interaction: TileInteraction::Walkable,
            action: TileAction::None,
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileInteraction {
    Block,
    Walkable,
    Swimmable,
    Crawl,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileAction {
    None,
    Destroyable,
    Death,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy)]
pub enum TileTexture {
    Grass,
    Water,
    ShallowWater,
    Sand,
    DeepWater,
    Dirt,
    Stone,
    SnowyMountain,
}

impl TileTexture {
    pub fn to_color(&self) -> Color {
        match self {
            TileTexture::Grass => GREEN,
            TileTexture::ShallowWater => SKYBLUE,
            TileTexture::Water => BLUE,
            TileTexture::DeepWater => DARKBLUE,
            TileTexture::Stone => GRAY,
            TileTexture::SnowyMountain => WHITE,
            TileTexture::Sand => YELLOW,
            TileTexture::Dirt => Color::from_rgba(155, 118, 83, 255),
        }
    }
}
