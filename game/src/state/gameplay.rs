#[derive(Default)]
pub struct GameplayState {
}

impl crate::store::SaveAndLoad for GameplayState {
    fn save(&self, _writer: &mut crate::store::SaveFileWriter) {
    }

    fn load(_reader: &mut crate::store::SaveFileReader) -> Self {
        GameplayState { }
    }
}
