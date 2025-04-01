use crate::error::Error;

pub struct SaveFileReader<'a> {
    pub data: &'a [u32],
    pub current_offset: usize,
    
}

impl<'a> SaveFileReader<'a> {
    pub fn new(bytes: &'a Box<[u8]>) -> Result<SaveFileReader<'a>, Error> {
        let data = Self::validate_buffer(bytes)?;
        let mut reader = SaveFileReader {
            data,
            current_offset: 0,
        };

        reader.validate_header()?;
        reader.current_offset = size_of::<super::SaveFileHeader>() / super::ALIGN;

        Ok(reader)
    }

    fn validate_buffer(bytes: &Box<[u8]>) -> Result<&[u32], Error> {
        let byte_slice: &[u8] = bytes.as_ref();
        if byte_slice.len() < size_of::<super::SaveFileHeader>() {
            return Err(save_err!("Data is smaller than the save file header size"));
        }

        let bytes_ptr = byte_slice.as_ptr() as usize;
        if bytes_ptr % 4 != 0 {
            return Err(save_err!("Data pointer is not aligned to 4 bytes"));
        }

        unsafe { Ok(byte_slice.align_to::<u32>().1) }
    }

    fn validate_header(&self) -> Result<(), Error> {
        let header_ptr = self.data.as_ptr() as *const super::SaveFileHeader;
        let header = unsafe { ::std::ptr::read(header_ptr) };
        
        if header.magic != super::MAGIC {
            return Err(save_err!("Decoder header magic does not match"));
        }

        if header.size != self.data.len() as u32 {
            return Err(save_err!("Header data size does not match buffer size"));
        }

        Ok(())
    }

    pub fn load<T: super::SaveAndLoad>(&mut self) -> T {
        T::load(self)
    }

    #[allow(dead_code)]
    pub fn load_option<T: super::SaveAndLoad>(&mut self) -> Option<T> {
        let option = self.read_u32();
        match option == 1 {
            true => Some(T::load(self)),
            false => None
        }
    }

    pub fn load_vec<T: super::SaveAndLoad>(&mut self) -> Vec<T> {
        let count = self.read_u32() as usize;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(T::load(self));
        }

        values
    }

    pub fn read<T: Copy>(&mut self) -> T {
        assert!(align_of::<T>() == super::ALIGN, "Alignment of T must be at least 4 bytes");
        let u32_count = size_of::<T>() / super::ALIGN;
        
        let data = unsafe {
            ::std::ptr::read(self.data.as_ptr().offset(self.current_offset as isize) as *const T)
        };

        self.current_offset += u32_count;

        data
    }

    pub fn read_u32(&mut self) -> u32 {
        let value = self.data[self.current_offset];
        self.current_offset += 1;
        value
    }

    pub fn read_f32(&mut self) -> f32 {
        let value = self.data[self.current_offset];
        self.current_offset += 1;
        f32::from_bits(value)
    }

    pub fn read_f64(&mut self) -> f64 {
        let top = self.data[self.current_offset];
        let bottom = self.data[self.current_offset + 1];
        self.current_offset += 2;

        f64::from_bits((bottom as u64) + ((top as u64) << 32))
    }

    pub fn read_u64(&mut self) -> u64 {
        let top = self.data[self.current_offset];
        let bottom = self.data[self.current_offset + 1];
        self.current_offset += 2;

        (bottom as u64) + ((top as u64) << 32)
    }

    pub fn read_slice<T: Copy>(&mut self) -> &[T] {
        assert!(align_of::<T>() == super::ALIGN, "Alignment of T must be {} bytes", super::ALIGN);

        let length = self.read_u32();
        if length == 0 {
            return &[];
        }

        let slice_length_bytes = size_of::<T>() * (length as usize);
        let remaining_buffer_size_bytes = (self.data.len() - self.current_offset) * super::ALIGN;
        assert!(remaining_buffer_size_bytes >= slice_length_bytes, "Not enough bytes left to read {length} instances of type");

        // Safety. Array will be large enough, but data might not be valid
        let data = unsafe {
            let data_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const T;
            ::std::slice::from_raw_parts(data_ptr, length as usize)
        };

        self.current_offset += slice_length_bytes / super::ALIGN;

        data
    }

    pub fn read_vec<T: Copy>(&mut self) -> Vec<T> {
        self.read_slice().to_vec()
    }

    pub fn read_bool_vec(&mut self) -> Vec<bool> {
        let length = self.read_u32();
        if length == 0 {
            return Vec::new();
        }

        let mut values = Vec::with_capacity(length as usize);
        for _ in 0..length {
            values.push(self.read_u32() == 1);
        }

        values
    }

    pub fn read_str(&mut self) -> &str {
        let length = self.read_u32();
        let length_padded = self.read_u32();

        let str = unsafe {
            let str_ptr = self.data.as_ptr().offset(self.current_offset as isize) as *const u8;
            let str_bytes = ::std::slice::from_raw_parts(str_ptr, length as usize);
            ::std::str::from_utf8(str_bytes).unwrap_or("UTF8 DECODING ERROR")
        };

        self.current_offset += (length_padded / 4) as usize;

        str
    }

    pub fn read_string_hashmap<T: Copy>(&mut self) -> fnv::FnvHashMap<String, T> {
        assert!(align_of::<T>() == super::ALIGN, "Alignment of T must be {} bytes", super::ALIGN);

        let mut out = fnv::FnvHashMap::default();
        let item_count = self.read_u32();
        if item_count == 0 {
            return out;
        }

        for _ in 0..item_count {
            let key = self.read_str().to_string();
            let value = self.read();
            out.insert(key, value);
        }

        out
    }
}
