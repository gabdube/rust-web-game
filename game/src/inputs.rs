use crate::shared::{Position, pos};

#[derive(Copy, Clone, Debug)]
pub enum Key {
    ShiftLeft
}

impl Key {
    pub fn from_name(name: &str) -> Option<Key> {
        match name {
            "ShiftLeft" => Some(Key::ShiftLeft),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ButtonState {
    Released = 0,
    JustReleased = 1,
    Pressed = 2,
    JustPressed = 3,
}

impl ButtonState {
    pub fn flip(&mut self) {
        match self {
            Self::JustPressed => { *self = Self::Pressed; }
            Self::JustReleased => { *self = Self::Released; }
            _ => {}
        }
    }

    pub fn pressed(self) -> bool {
        self == Self::JustPressed || self == Self::Pressed
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left  = 0,
    Right = 1
}

pub struct InputState {
    pub last_mouse_position: Position<f32>,
    pub mouse_position: Position<f32>,
    pub mouse_buttons: [ButtonState; 2],
    pub left_shift: ButtonState,
}

impl InputState {

    pub fn right_mouse_clicked(&self) -> bool {
        self.mouse_buttons[MouseButton::Right as usize] == ButtonState::JustPressed
    }

    pub fn right_mouse_released(&self) -> bool {
        self.mouse_buttons[MouseButton::Right as usize] == ButtonState::JustReleased
    }

    pub fn left_mouse_clicked(&self) -> bool {
        self.mouse_buttons[MouseButton::Left as usize] == ButtonState::JustPressed
    }

    pub fn mouse_delta(&self) -> Option<Position<f32>> {
        let delta = self.mouse_position - self.last_mouse_position;
        if delta.x != 0.0 || delta.y != 0.0 {
            Some(delta)
        } else {
            None
        }
    }

    //
    // Updates
    //

    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position.x = x;
        self.mouse_position.y = y;
    }

    pub fn update_mouse_buttons(&mut self, button: MouseButton, pressed: ButtonState) {
        let index = button as usize;
        self.mouse_buttons[index] = pressed;
    }

    pub fn update_keys(&mut self, key: Key, pressed: ButtonState) {
        match key {
            Key::ShiftLeft => { self.left_shift = pressed; }
        }
    }

}


//
// Other Impls
//

impl Default for InputState {
    fn default() -> Self {
        InputState {
            last_mouse_position: pos(0.0, 0.0),
            mouse_position: pos(0.0, 0.0),
            mouse_buttons: [ButtonState::Released; 2],
            left_shift: ButtonState::Released,
        }
    }
}

impl TryFrom<u8> for MouseButton {
    type Error = crate::error::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseButton::Left),
            1 => Ok(MouseButton::Right),
            _ => { Err(undefined_err!("{value} is not a valid mouse button identifier"))}
        }
    }
}
