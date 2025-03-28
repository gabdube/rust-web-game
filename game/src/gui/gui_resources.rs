use std::marker::PhantomData;
use crate::assets::TextMetrics;
use crate::shared::AABB;

pub type GuiImageId = GuiResourceId<GuiImage>;
pub type GuiStaticTextId = GuiResourceId<TextMetrics>;

/// Id representing a resource type in the gui
pub struct GuiResourceId<T> {
    value: u32,
    dynamic_id: u32,
    _t: PhantomData<T>,
}

impl<T> GuiResourceId<T> {
    pub const fn new(index: usize) -> Self {
        GuiResourceId { 
            value: index as u32,
            dynamic_id: u32::MAX, 
            _t: PhantomData
        }
    }

    pub const fn new_dyn(index: usize, dyn_index: usize) -> Self {
        GuiResourceId { 
            value: index as u32,
            dynamic_id: dyn_index as u32, 
            _t: PhantomData
        }
    }

    pub const fn index(&self) -> usize {
        self.value as usize
    }

    pub const fn dyn_index(&self) -> usize {
        self.dynamic_id as usize
    }

    pub const fn is_dyn(&self) -> bool {
        self.dynamic_id != u32::MAX
    }
}

#[derive(Copy, Clone, Default)]
pub struct GuiImage {
    pub texcoord: AABB,
}

impl GuiImage {
    pub const fn from_aabb(texcoord: AABB) -> Self {
        GuiImage { texcoord }
    }
}

pub struct DynamicResource {
    /// List of component index using this resource
    pub users: Vec<u32>
}

impl Default for DynamicResource {
    fn default() -> Self {
        DynamicResource { users: Vec::with_capacity(2) }
    }
}

impl crate::store::SaveAndLoad for DynamicResource {
    fn save(&self, writer: &mut crate::store::SaveFileWriter) {
        writer.write_slice(&self.users);
    }

    fn load(reader: &mut crate::store::SaveFileReader) -> Self {
        DynamicResource { 
            users: reader.read_vec(),
        }
    }
}

impl<T> Default for GuiResourceId<T> {
    fn default() -> Self {
        GuiResourceId { value: u32::MAX, dynamic_id: u32::MAX, _t: PhantomData }
    }
}

impl<T> Clone for GuiResourceId<T> {
    fn clone(&self) -> Self {
        GuiResourceId { value: self.value, dynamic_id: self.dynamic_id, _t: PhantomData }
    }
}

impl<T> Copy for GuiResourceId<T> {

}
