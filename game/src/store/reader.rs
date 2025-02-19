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

    pub fn read_slice<T: Copy>(&mut self) -> &[T] {
        let align = align_of::<T>();
        assert!(align == super::ALIGN, "Alignment of T must be {} bytes", super::ALIGN);

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
}
