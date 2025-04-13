#[cfg(feature="debug")]
pub mod debug_state;

#[cfg(feature="debug")]
pub use debug_state::*;

/// Empty debug state for if the debug feature flags is disabled
#[cfg(not(feature="debug"))]
#[allow(dead_code)]
#[derive(Default)]
pub struct DebugState {}

#[cfg(not(feature="debug"))]
impl crate::store::SaveAndLoad for DebugState {
    fn save(&self, _writer: &mut crate::store::SaveFileWriter) {
    }

    fn load(_reader: &mut crate::store::SaveFileReader) -> Self {
        DebugState {}
    }
}
