use std::{ops, fmt::Debug};
use crate::store::SaveAndLoad;

#[derive(Copy, Clone, PartialEq)]
pub struct Position<T: Copy> {
    pub x: T,
    pub y: T,
}

impl Position<f32> {

    pub fn distance(&self, other: Position<f32>) -> f32 {
        let x2 = other.x - self.x;
        let y2 = other.y - self.y;
        f32::sqrt(x2*x2 + y2*y2)
    }

}

impl SaveAndLoad for Position<f32> {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f32(self.x);
        writer.write_f32(self.y);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Position {
            x: reader.read_f32(),
            y: reader.read_f32(),
        }
    }
}

impl ops::Add for Position<f32> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Sub for Position<f32> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ops::SubAssign for Position<f32> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T: Debug+Copy> Debug for Position<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Position")
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}

impl<T: Copy+Default> Default for Position<T> {
    fn default() -> Self {
        let v = Default::default();
        Position { x: v, y: v }
    }
}



#[derive(Copy, Clone, PartialEq)]
pub struct Size<T: Copy> {
    pub width: T,
    pub height: T,
}

impl<T: Default+Copy> Default for Size<T> {
    fn default () -> Self {
        let v = Default::default();
        Size { width: v, height: v }
    }
}

impl<T: Debug+Copy> Debug for Size<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Size")
            .field(&self.width)
            .field(&self.height)
            .finish()
    }
}

impl SaveAndLoad for Size<f32> {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_f32(self.width);
        writer.write_f32(self.height);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        Size {
            width: reader.read_f32(),
            height: reader.read_f32(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct AABB {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

impl AABB {

    pub const fn from_position_and_size(pos: Position<f32>, size: Size<f32>) -> Self {
        AABB {
            left: pos.x,
            top: pos.y,
            right: pos.x + size.width,
            bottom: pos.y + size.height,
        }
    }

    pub const fn splat(&self) -> [f32; 4] {
        [self.left, self.top, self.right, self.bottom]
    }

    pub const fn width(&self) -> f32 {
        self.right - self.left
    }

    pub const fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub const fn size(&self) -> Size<f32> {
        Size { width: self.width(), height: self.height() }
    }
   
    pub fn offset(&mut self, offset: Position<f32>) {
        self.left += offset.x;
        self.right += offset.x;
        self.top += offset.y;
        self.bottom += offset.y;
    }

    pub const fn intersects(&self, other: &Self) -> bool {
        if self.right < other.left || other.right < self.left {
            return false
        }

        if self.bottom < other.top || other.bottom < self.top {
            return false
        }

        true
    }

    pub const fn point_inside(&self, point: Position<f32>) -> bool {
        point.x >= self.left && point.x <= self.right && point.y >= self.top && point.y <= self.bottom
    }

}


//
// Helpers method
//

pub const fn pos<T:Copy>(x: T, y: T) -> Position<T> {
    Position { x, y }
}

pub const fn size<T:Copy>(width: T, height: T) -> Size<T> {
    Size { width, height }
}

pub const fn aabb(position: Position<f32>, size: Size<f32>) -> AABB {
    AABB {
        left: position.x,
        top: position.y,
        right: position.x + size.width,
        bottom: position.y + size.height
    }
}

/// Split a csv string into up to `MAX_ARGS` parameters. Calls `callback` for each line splitted.
pub fn split_csv<const MAX_ARGS: usize, CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
    let mut start = 0;
    let mut end = 0;
    let mut chars_iter = csv.chars();
    let mut args: [&str; MAX_ARGS] = [""; MAX_ARGS];
    while let Some(c) = chars_iter.next() {
        end += 1;
        if c == '\n' {
            let line = &csv[start..end];
            let mut args_count = 0;
            for substr in line.split(';') {
                if args_count < MAX_ARGS {
                    args[args_count] = substr;
                    args_count += 1;
                }
            }

            if args_count > 1 {
                callback(&args[0..args_count]);
            }

            start = end;
        }
    }
}

pub fn merge_error(err: &mut Option<crate::error::Error>, new: crate::error::Error) {
    if err.is_none() {
        *err = Some(new);
    } else {
        err.as_mut().unwrap().merge(new);
    }
}
