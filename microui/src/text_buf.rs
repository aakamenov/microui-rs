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
        if len > self.free_space() {
            return Err(fmt::Error);
        }

        self.push_str(s);

        Ok(())
    }
}

impl<const N: usize> TextBuf for ConstStr<N> {
    fn push_str(&mut self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let free = self.free_space();
        let bytes = text.as_bytes();

        let count = if bytes.len() > free {
            let mut len = free;

            while len > 0 {
                // Check if the byte is a character boundary.
                // Based on std: https://github.com/rust-lang/rust/blob/bbdca4c28fd9b57212cb3316ff4ffb1529affcbe/library/core/src/num/mod.rs#L883
                if (bytes[len] as i8) >= -0x40 {
                    break;
                }

                len -= 1;
            }

            len
        } else {
            bytes.len()
        };

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

            if (self[len] as i8) >= -0x40 {
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

impl<const N: usize> Into<String> for ConstStr<N> {
    fn into(self) -> String {
        self.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;
    use super::*;

    #[test]
    fn pop_char() {
        let mut vec = ConstStr::<10>::new();
        vec.push_str("ß");
        vec.push_str("X");
        vec.push_str("東");
        vec.push_str("💩");

        let text = "ßX東💩";
        assert_eq!(vec.as_str(), text);

        while vec.len() > 0 {
            vec.pop_char();
            
            let slice = &text[0..vec.len()];
            assert_eq!(vec.as_str(), slice);
        }

        assert_eq!(vec.len(), 0);
        assert_eq!(vec.as_str(), "");
    }

    #[test]
    fn write_str() {
        let mut vec = ConstStr::<10>::new();
        vec.write_str(&"X".repeat(10)).unwrap();
        vec.write_str(&"X").unwrap_err();
    }

    #[test]
    fn push_str() {
        let mut vec = ConstStr::<10>::new();
        let chars = "XXXXXXß";

        assert_eq!(vec.push_str(chars), 8);
        assert_eq!(vec.len(), 8);
        assert_eq!(vec.as_str(), chars);

        assert_eq!(vec.push_str(""), 0);
        assert_eq!(vec.len(), 8);
        assert_eq!(vec.as_str(), chars);

        assert_eq!(vec.push_str("東X"), 0);
        assert_eq!(vec.len(), 8);
        assert_eq!(vec.as_str(), chars);

        assert_eq!(vec.push_str("ß東"), 2);
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.as_str(), [chars, "ß"].concat());

        assert_eq!(vec.push_str("ß"), 0);
        assert_eq!(vec.len(), 10);
        assert_eq!(vec.as_str(), [chars, "ß"].concat());

        vec.pop_char();
        assert_eq!(vec.len(), 8);
        assert_eq!(vec.as_str(), chars);

        assert_eq!(vec.push_str("Xß"), 1);
        assert_eq!(vec.len(), 9);
        assert_eq!(vec.as_str(), [chars, "X"].concat());

        assert_eq!(vec.push_str("東"), 0);
        assert_eq!(vec.len(), 9);
        assert_eq!(vec.as_str(), [chars, "X"].concat());
    }
}
