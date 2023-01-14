use std::{
    fmt,
    hash::{Hash, Hasher},
    ops::BitXor
};

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Id(pub u64);

impl Id {
    pub fn new(item: &impl Hash, entropy: u64) -> Self {
        let mut hasher = Fnv1a::new(entropy);
        item.hash(&mut hasher);

        Self(hasher.finish())
    }
}

// 64 bit FNV-1a hash as described here:
// http://www.isthe.com/chongo/tech/comp/fnv/#FNV-1a
struct Fnv1a(u64);

impl Fnv1a {
    const OFFSET_BASIS: u64 = 14695981039346656037;

    fn new(entropy: u64) -> Self {
        let entropy = if entropy > 0 {
            entropy
        } else {
            Self::OFFSET_BASIS
        };

        Self(entropy)
    }
}

impl Hasher for Fnv1a {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for i in 0..bytes.len() {
            self.0 = self.0.bitxor(bytes[i] as u64).wrapping_mul(1099511628211);
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
