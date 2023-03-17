use macroquad::{
    prelude::{
        color_u8, draw_circle, is_mouse_button_down, mouse_position, touches, vec2, Color,
        MouseButton, TouchPhase, Vec2,
    },
    shapes::draw_rectangle,
    time::get_frame_time,
};

pub struct Button {
    pos: Vec2,
    size: Vec2,
    event: ButtonEvent,
    // touch_id: Option<u64>,
    drawable: Box<dyn Fn(f32, f32, f32, f32)>
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
    Hold(f32),
    Pressed(f32),
    Inactive,
}

impl Button {
    pub fn rectangle(pos: Vec2, size: Vec2) -> Button {
        let background_fn = Box::new(|x: f32, y: f32, w: f32, h: f32| {
            draw_rectangle(x, y, w, h, color_u8!(96, 128, 144, 128))
        });
        Button {
            pos: pos - size/2.,
            size,
            event: ButtonEvent::Inactive,
            // touch_id: None,
            drawable: background_fn
        }
    }
    pub fn circle(pos: Vec2, r: f32) -> Button {
        let background_fn = Box::new(|x: f32, y: f32, w: f32, _h: f32| {
            draw_circle(x+w/2., y+w/2., w/2., color_u8!(96, 128, 144, 128));
        });
        Button {
            pos: pos - r/2.,
            size: vec2(r, r),
            event: ButtonEvent::Inactive,
            // touch_id: None,
            drawable: background_fn
        }
    }

    pub fn update(&mut self) -> ButtonEvent {
        if touches().is_empty() {
            self.update_mouse();
        } else {
            self.update_touch();
        }
        self.event
    }

    pub fn pressed(&mut self) -> bool {
        match self.event {
            ButtonEvent::Pressed(_) => true,
            _ => false,
        }
    }

    pub fn down(&mut self) -> bool {
        match self.event {
            ButtonEvent::Hold(_) | ButtonEvent::Pressed(_) => true,
            _ => false,
        }
    }

    pub fn render(&self) {
        (self.drawable)(self.pos.x, self.pos.y, self.size.x,self.size.y);
    }

    fn update_mouse(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        let mouse = Vec2::new(mouse_x, mouse_y);
        let pressed = is_mouse_button_down(MouseButton::Left);
        let in_box = in_box(self.pos, self.size, mouse);
        self.event = match (pressed, in_box, self.event) {
            (true, true, ButtonEvent::Hold(time)) => ButtonEvent::Hold(time + get_frame_time()),
            (true, true, ButtonEvent::Inactive) => ButtonEvent::Hold(0.),
            (_, _, ButtonEvent::Hold(time)) => ButtonEvent::Pressed(time),
            _ => ButtonEvent::Inactive,
        };
    }

    fn update_touch(&mut self) {
        for touch in touches() {
            let in_box = in_box(self.pos, self.size, touch.position);
            let touch_id: Option<u64> = None;
            match (touch.phase, touch_id, in_box, self.event) {
                (_, _, _, ButtonEvent::Pressed(_)) => {
                    // self.touch_id = None;
                    self.event = ButtonEvent::Inactive;
                }
                (TouchPhase::Started, None, true, _) => {
                    // self.touch_id = Some(touch.id);
                    self.event = ButtonEvent::Hold(0.)
                }
                (TouchPhase::Stationary, _, true, ButtonEvent::Hold(time))
                | (TouchPhase::Moved, _, true, ButtonEvent::Hold(time)) => {
                    // if touch_id == touch.id {
                        self.event = ButtonEvent::Hold(time + get_frame_time());
                    // }
                }
                (TouchPhase::Ended, _, _, ButtonEvent::Hold(time))
                | (TouchPhase::Cancelled, _, _, ButtonEvent::Hold(time)) => {
                    // if touch_id == touch.id {
                        self.event = ButtonEvent::Pressed(time);
                        // self.touch_id = None;
                    // }
                }
                _ => (),
            }
        }
    }
}

fn in_box(pos: Vec2, size: Vec2, point: Vec2) -> bool {
    let x_in_range = point.x >= pos.x && point.x <= pos.x + size.x;
    let y_in_range = point.y >= pos.y && point.y <= pos.y + size.y;
    x_in_range && y_in_range
}
