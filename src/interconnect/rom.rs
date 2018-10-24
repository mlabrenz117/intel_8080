use std::convert::From;

pub struct Rom {
    bytes: Box<[u8]>,
    //ptr: *mut u8,
}

impl Rom {
    pub(crate) fn read_byte(&self, addr: u16) -> u8 {
        self.bytes[addr as usize]
        //  let mask = (self.bytes.len() - 1) as u16;
        //  let addr = addr & mask;
        //  unsafe { *self.ptr.offset(addr as isize) }
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }
}

impl<T> From<T> for Rom
where
    T: AsRef<[u8]>,
{
    fn from(data: T) -> Rom {
        let bytes: Box<[u8]> = Box::from(data.as_ref());
        //let ptr = bytes.as_mut_ptr();
        Rom { bytes }
    }
}
