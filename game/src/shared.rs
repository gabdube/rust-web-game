use std::fmt::Debug;
use crate::store::SaveAndLoad;

#[derive(Copy, Clone)]
pub struct Position<T: Copy> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Position<T> {
    pub const fn splat(self) -> [T; 2] {
        [self.x, self.y]
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

pub fn pos<T:Copy>(x: T, y: T) -> Position<T> {
    Position { x, y }
}
