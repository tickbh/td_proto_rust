use std::io::{Read, Write, Result};
use std::ptr;
use std::fmt;

pub struct Buffer {
    val   : Vec<u8>,
    rpos  : usize,
    wpos  : usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            val   : Vec::new(),
            rpos  : 0,
            wpos  : 0,
        }
    }

    pub fn len(&self) -> usize {
        self.val.len()
    }

    pub fn set_rpos(&mut self, rpos : usize) {
        self.rpos = rpos;
    }

    pub fn get_rpos(&self) -> usize {
        self.rpos
    }

    pub fn set_wpos(&mut self, wpos : usize) {
        self.wpos = wpos;
    }

    pub fn get_wpos(&self) -> usize {
        self.wpos
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "bytes ({:?})", self.val)
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let left = self.val.len() - self.rpos;
        if left == 0 {
            return Ok(0);
        }
        let read = if left > buf.len() { buf.len() } else { left };
        unsafe { ptr::copy(&self.val[self.rpos], &mut buf[0], read); }
        self.rpos += read;
        Ok(read)
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        if self.val.len() < self.wpos + buf.len() {
            self.val.resize(self.wpos + buf.len(), 0);
        }
        unsafe { ptr::copy(&buf[0], &mut self.val[self.wpos], buf.len()); }
        self.wpos += buf.len();
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
