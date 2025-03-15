use super::Action;

/// Temporary buffer to store new actions
pub struct ActionsBuffer {
    pub new: Vec<Action>,
}

impl ActionsBuffer {
    pub fn push(&mut self, action: Action) {
        self.new.push(action);
    }
}

impl Default for ActionsBuffer {
    fn default() -> Self {
        ActionsBuffer {
            new: Vec::with_capacity(16),
        }
    }
}

impl crate::store::SaveAndLoad for ActionsBuffer {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.new);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        ActionsBuffer {
            new: reader.read_slice().to_vec(),
        }
    }
}
