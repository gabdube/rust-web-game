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

    pub fn save<T: super::SaveAndLoad>(&mut self, value: &T) {
        value.save(self);
    }

    pub fn save_option<T: super::SaveAndLoad>(&mut self, value: &Option<T>) {
        match value.as_ref() {
            Some(value) => {
                self.write_u32(1);
                value.save(self);
            },
            None => {
                self.write_u32(0);
            }
        }
    }

    pub fn save_slice<T: super::SaveAndLoad>(&mut self, values: &[T]) {
        self.write_u32(values.len() as u32);
        
        if values.len() == 0 {
            // 0 for the size and no data
            return;
        }

        for value in values {
            value.save(self);
        }
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

    pub fn write_f32(&mut self, data: f32) {
        self.try_realloc(1);
        self.write_u32_inner(data.to_bits());
    }

    pub fn write_f64(&mut self, data: f64) {
        self.try_realloc(2);
        
        let bits = data.to_bits();
        self.write_u32_inner((bits >> 32) as u32);
        self.write_u32_inner(bits as u32);
    }

    pub fn write_slice<T: Copy>(&mut self, values: &[T]) {
        let align = align_of::<T>();
        assert!(align == super::ALIGN, "Data align must be {}", super::ALIGN);

        if values.len() == 0 {
            // 0 for the size and no data
            self.write_u32(0);
            return;
        }

        let (_, aligned, _) = unsafe { values.align_to::<u32>() };
        self.try_realloc(aligned.len() + 1);

        self.write_u32_inner(values.len() as u32);

        for &value in aligned {
            self.write_u32_inner(value);
        }
    }

    pub fn write_bool_slice(&mut self, values: &[bool]) {
        if values.len() == 0 {
            // 0 for the size and no data
            self.write_u32(0);
            return;
        }

        let count = values.len();
        self.try_realloc(count + 1);

        self.write_u32_inner(count as u32);

        for &value in values {
            self.write_u32_inner(value as u32);
        }
    }

    pub fn write_str(&mut self, value: &str) {
        let padding = 4 - (value.len() % 4);
        let length = value.len();
        let padded_length = length + padding;

        let u32_count = padded_length / 4;
        self.try_realloc(u32_count + 2);
        
        self.write_u32_inner(length as u32);
        self.write_u32_inner(padded_length as u32);

        unsafe { 
            ::std::ptr::copy_nonoverlapping::<u8>(
                value.as_ptr(),
                self.data.as_ptr().offset(self.data_offset as isize) as *mut u8,
                length as usize
            );
        }

        self.data_offset += u32_count as u32;
    }

    pub fn write_string_hashmap<T: Copy>(&mut self, data: &fnv::FnvHashMap<String, T>) {
        assert!(align_of::<T>() == super::ALIGN, "Data alignment must be 4 bytes");

        if data.len() == 0 {
            // 0 for the size and no data
            self.write_u32(0);
            return;
        }

        self.try_realloc(1);
        self.write_u32_inner(data.len() as u32);

        for (key, value) in data.iter() {
            self.write_str(key);
            self.write(value);
        }
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
