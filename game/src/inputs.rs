use crate::shared::{Position, Size, pos, size};

#[derive(Copy, Clone, Debug)]
pub enum Key {
    CtrlLeft
}

impl Key {
    pub fn from_name(name: &str) -> Option<Key> {
        match name {
            "ControlLeft" => Some(Key::CtrlLeft),
            _ => None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone)]
pub enum MouseButton {
    Left  = 0,
    Right = 1,
    Center = 2,
}

#[derive(Copy, Clone)]
pub struct InputState {
    pub last_view_size: Size<f32>,
    pub view_size: Size<f32>,
    pub last_mouse_position: Position<f32>,
    pub mouse_position: Position<f32>,
    pub mouse_buttons: [ButtonState; 3],
    pub left_ctrl: ButtonState,
}

impl InputState {

    pub fn mouse_button_state(&self, button: MouseButton) -> ButtonState {
        self.mouse_buttons[button as usize]
    }

    pub fn right_mouse_clicked(&self) -> bool {
        self.mouse_button_state(MouseButton::Right) == ButtonState::JustPressed
    }

    pub fn left_mouse_clicked(&self) -> bool {
        self.mouse_button_state(MouseButton::Left) == ButtonState::JustPressed
    }

    pub fn mouse_delta(&self) -> Option<Position<f32>> {
        let delta = self.mouse_position - self.last_mouse_position;
        if delta.x != 0.0 || delta.y != 0.0 {
            Some(delta)
        } else {
            None
        }
    }

    pub fn view_resized(&self) -> bool {
        self.last_view_size != self.view_size
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
            Key::CtrlLeft => { self.left_ctrl = pressed; }
        }
    }

}


//
// Other Impls
//

impl Default for InputState {
    fn default() -> Self {
        InputState {
            last_view_size: size(0.0, 0.0),
            view_size: size(0.0, 0.0),
            last_mouse_position: pos(0.0, 0.0),
            mouse_position: pos(0.0, 0.0),
            mouse_buttons: [ButtonState::Released; 3],
            left_ctrl: ButtonState::Released,
        }
    }
}

impl TryFrom<u8> for MouseButton {
    type Error = crate::error::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MouseButton::Left),
            1 => Ok(MouseButton::Right),
            2 => Ok(MouseButton::Center),
            _ => { Err(undefined_err!("{value} is not a valid mouse button identifier"))}
        }
    }
}
