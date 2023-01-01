use macroquad::prelude::*;

pub trait Entity {
    fn get_velocity(&self) -> Vec2;
    fn get_position(&self) -> Vec2;
    fn set_position(&mut self, pos: Vec2);
}

pub struct Player {
    pos: Vec2,
    v: Vec2,
    color: Color
}

impl Player {
    pub fn new(x: f32, y: f32,) -> Player{
        Player{
            pos: vec2(x, y),
            v: vec2(0., 0.),
            color: PURPLE
        }
    }
    pub fn set_color(&mut self, color: Color){
        self.color = color;
    }
    pub fn render(&self) {
        const SIZE: f32 = 1.;
        draw_circle(self.pos.x, self.pos.y, SIZE, self.color);
    }
    pub fn set_velocity(&mut self, velocity: Vec2){
        self.v = velocity;
    }
}

impl Entity for Player {
    fn get_velocity(&self) -> Vec2 {
        self.v
    }

    fn get_position(&self) -> Vec2 {
        self.pos
    }

    fn set_position(&mut self, pos: Vec2) {
        self.pos = pos;
    }

}