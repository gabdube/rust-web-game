use crate::shared::{Position, pos};

#[repr(u8)]
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
