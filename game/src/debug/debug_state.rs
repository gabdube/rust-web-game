#![allow(dead_code)]
use crate::shared::{AABB, Position};

#[derive(Copy, Clone)]
pub enum DebugElement {
    Rect { base: AABB, color: [u8; 4] },
    Line { start: Position<f32>, end: Position<f32>, color: [u8; 4] },
    Triangle { v0: Position<f32>, v1: Position<f32>, v2: Position<f32>, color: [u8; 4] },
    Point { pt: Position<f32>, size: f32, color: [u8; 4] }
}

/// Hold debugging information to be displayed on screen
/// This requires the feature "debug"
/// Debugging info is cleared every frame
pub struct DebugState {
    pub elements: Vec<DebugElement>
}

impl DebugState {
    
    pub fn debug_rect(&mut self, base: AABB, color: [u8; 4]) {
        self.elements.push(DebugElement::Rect { base, color })
    }

    pub fn debug_triangle(&mut self, v0: Position<f32>, v1: Position<f32>, v2: Position<f32>, color: [u8; 4]) {
        self.elements.push(DebugElement::Triangle { v0, v1, v2, color })
    }

    pub fn debug_line(&mut self, start: Position<f32>, end: Position<f32>, color: [u8; 4]) {
        self.elements.push(DebugElement::Line { start, end, color })
    }

    pub fn debug_point(&mut self, pt: Position<f32>, size: f32, color: [u8; 4]) {
        self.elements.push(DebugElement::Point { pt, size, color });
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

}

impl Default for DebugState {
    fn default() -> Self {
        DebugState {
            elements: Vec::with_capacity(16)
        }
    }
}
