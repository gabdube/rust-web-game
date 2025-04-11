use crate::shared::AABB;

#[derive(Copy, Clone)]
pub enum DebugElement {
    Rect { base: AABB, color: [u8; 4] }
}

/// Hold debugging information to be displayed on screen
/// This requires the feature "debug"
/// Debugging info is cleared every frame
pub struct DebugState {
    pub elements: Vec<DebugElement>
}

impl DebugState {

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

impl crate::store::SaveAndLoad for DebugState {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.elements);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        let elements = reader.read_vec();
        DebugState {
            elements
        }
    }
}
