use std::fmt::Debug;
use crate::store::SaveAndLoad;

#[derive(Copy, Clone)]
pub struct Position<T: Copy> {
    pub x: T,
    pub y: T,
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


#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32
}

impl AABB {

    pub fn intersects(&self, other: &Self) -> bool {
        if self.right < other.left || other.right < self.left {
            return false
        }

        if self.bottom < other.top || other.bottom < self.top {
            return false
        }

        true
    }

}

//
// Helpers method
//

pub fn pos<T:Copy>(x: T, y: T) -> Position<T> {
    Position { x, y }
}

pub fn aabb(position: Position<f32>, size: Size<f32>) -> AABB {
    AABB {
        left: position.x,
        top: position.y,
        right: position.x + size.width,
        bottom: position.y + size.height
    }
}


pub fn split_csv<CB: FnMut(&[&str])>(csv: &str, mut callback: CB) {
    let mut start = 0;
    let mut end = 0;
    let mut chars_iter = csv.chars();
    let mut args: [&str; 8] = [""; 8];
    while let Some(c) = chars_iter.next() {
        end += 1;
        if c == '\n' {
            let line = &csv[start..end];
            let mut args_count = 0;
            for substr in line.split(';') {
                args[args_count] = substr;
                args_count += 1;
            }

            if args_count > 1 {
                callback(&args[0..(args_count-1)]);
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
