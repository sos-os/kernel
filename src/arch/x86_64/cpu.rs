use ::{io,util};

pub struct Port(u16);

impl Port {
    /// Read a byte (8 bits) from this port
    pub unsafe fn in8(&self) -> u8 {
        let result: u8;
        asm!(  "in al, dx"
            :  "={al}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }
    /// Read a word (16 bits) from this port
    pub unsafe fn in16(&self) -> u16 {
        let result: u16;
        asm!(  "in ax, dx"
            :  "={ax}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }

    /// Read a long word (32 bits) from this port
    pub unsafe fn in32(&self) -> u32 {
        let result: u32;
        asm!(  "in eax, dx"
            :  "={eax}"(result)
            :  "{dx}"(self.0)
            :: "intel"
             , "volatile" );
        result
    }

    pub unsafe fn out8(&self, value: u8) {
         asm!(  "out dx, al"
             :: "{dx}"(self.0)
              , "{al}"(value)
             :: "intel"
              , "volatile" );
    }

    pub unsafe fn out16(&self, value: u16) {
         asm!(  "out dx, ax"
             :: "{dx}"(self.0)
              , "{ax}"(value)
             :: "intel"
              , "volatile" );
    }

    pub unsafe fn out32(&self, value: u32) {
         asm!(  "out dx, eax"
             :: "{dx}"(self.0)
              , "{eax}"(value)
             :: "intel"
              , "volatile" );
    }
}

impl io::Read for Port {
    type Error = util::Void;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        unsafe {
            Ok(match &mut *buf {
                []                  => 0 // no bytes were read
              , [ref mut head, _..] => { *head = self.in8(); 1 }
            })
        }
    }

    fn read_all(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_bytes = 0;
        for idx in buf.iter_mut() {
            unsafe { *idx = self.in8();
                     read_bytes += 1; }
        }
        Ok(read_bytes)
    }

}
