use std::{ptr, fmt};

use crate::const_vec::ConstStr;

pub trait TextBuf: fmt::Write {
    fn as_str(&self) -> &str;
    fn push_str(&mut self, text: &str) -> usize;
    fn pop_char(&mut self);
}

impl<const N: usize> fmt::Write for ConstStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let len = s.as_bytes().len();

        // This method can only succeed if the entire string slice was successfully written.
        if len > self.capacity() - self.len() {
            return Err(fmt::Error);
        }

        self.push_str(s);

        Ok(())
    }
}

impl<const N: usize> TextBuf for ConstStr<N> {
    fn push_str(&mut self, text: &str) -> usize {
        let bytes = text.as_bytes();
        let count = bytes.len().clamp(0, self.capacity() - self.len());

        if count > 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.ptr_at_mut(self.len()),
                    count
                );
    
                self.set_len(self.len() + count);
            }
        }

        count
    }

    fn pop_char(&mut self) {
        let mut len = self.len();

        // Skip utf-8 continuation bytes (multi-byte characters).
        while len > 0 {
            len -= 1;

            if self[len] & 0xc0 != 0x80 {
                break;
            }
        }

        unsafe { self.set_len(len) }
    }

    #[inline]
    fn as_str(&self) -> &str {
        // SAFETY: All usage through this interface guarantees that the bytes
        // stored are always valid utf-8 because &str is always valid utf-8
        unsafe { std::str::from_utf8_unchecked(self.as_ref()) }
    }
}

impl TextBuf for String {
    #[inline]
    fn push_str(&mut self, text: &str) -> usize {
        self.push_str(text);

        text.as_bytes().len()
    }

    #[inline]
    fn pop_char(&mut self) {
        self.pop();
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self
    }
}
