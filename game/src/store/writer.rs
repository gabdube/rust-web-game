pub struct SaveFileWriter {
    pub data: Vec<u32>,
    pub data_offset: u32,
}

impl SaveFileWriter {
    pub fn new() -> Self {
        let mut writer = SaveFileWriter {
            data: vec![0; 3000],
            data_offset: 0,
        };

        writer.write_header();

        writer
    }

    pub fn finalize(mut self) -> Vec<u8> {
        let total_size = self.data_offset;

        let offset: usize = ::std::mem::offset_of!(super::SaveFileHeader, size) / super::ALIGN;
        self.data[offset] = total_size;

        let total_size = total_size as usize;
        let total_size_bytes = total_size * super::ALIGN;
        let mut out_bytes = vec![0u8; total_size_bytes];
        unsafe { ::std::ptr::copy_nonoverlapping::<u32>(self.data.as_ptr(), out_bytes.as_mut_ptr() as *mut u32, total_size); }

        out_bytes
    }

    fn write_header(&mut self) {
        self.write(&super::SaveFileHeader::new());        
    }

    pub fn write<T: Copy>(&mut self, data: &T) {
        assert!(align_of::<T>() == super::ALIGN, "Data alignment must be 4 bytes");

        let data_array = ::std::slice::from_ref(data);
        let (_, aligned, _) = unsafe { data_array.align_to::<u32>() };

        let u32_count = aligned.len();
        self.try_realloc(u32_count);

        for &value in aligned {
            self.write_u32_inner(value);
        }
    }

    pub fn write_u32(&mut self, data: u32) {
        self.try_realloc(1);
        self.write_u32_inner(data);
    }

    #[inline(always)]
    fn write_u32_inner(&mut self, value: u32) {
        self.data[self.data_offset as usize] = value;
        self.data_offset += 1;
    }

    fn try_realloc(&mut self, data_count: usize) {
        let data_offset = self.data_offset as usize;
        if data_offset + data_count >= self.data.len() {
            self.data.reserve_exact(2000 + data_count);
            unsafe { self.data.set_len(self.data.capacity()); }
        }
    }
}
