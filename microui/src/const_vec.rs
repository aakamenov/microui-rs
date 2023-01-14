use std::{
    mem::{self, MaybeUninit},
    cmp::Ordering,
    ops::{Index, IndexMut},
    convert::AsRef,
    fmt,
    any
};

pub type ConstStr<const N: usize> = ConstVec<u8, N>;

pub struct ConstVec<T, const N: usize> {
    index: usize,
    items: [MaybeUninit<T>; N]
}

impl<T, const N: usize> ConstVec<T, N> {
    pub fn new() -> Self {
        unsafe {
            Self {
                index: 0,
                items: MaybeUninit::uninit().assume_init()
            }
        }
    }

    pub fn new_zeroed() -> Self {
        unsafe {
            Self {
                index: 0,
                items: MaybeUninit::zeroed().assume_init()
            }
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        assert!(
            self.index < N,
            "Ran out of memory in ConstVec<{}, {}>",
            any::type_name::<T>(),
            N
        );

        self.items[self.index].write(item);
        self.index += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.index == 0 {
            None
        } else {
            self.index -= 1;

            unsafe {
                Some(self.items[self.index].assume_init_read())
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        if mem::needs_drop::<T>() {
            for item in &mut self.items[0..self.index] {
                unsafe {
                    item.assume_init_drop();
                }
            }
        }

        self.index = 0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn free_space(&self) -> usize {
        self.capacity() - self.len()
    }

    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.index == 0 {
            None
        } else {
            unsafe {
                Some(self.items[self.index - 1].assume_init_ref())
            }
        }
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.index == 0 {
            None
        } else {
            unsafe {
                Some(self.items[self.index - 1].assume_init_mut())
            }
        }
    }

    #[inline]
    pub fn first(&mut self) -> Option<&T> {
        if self.index == 0 {
            None
        } else {
            unsafe {
                Some(self.items[0].assume_init_ref())
            }
        }
    }

    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.index == 0 {
            None
        } else {
            unsafe {
                Some(self.items[0].assume_init_mut())
            }
        }
    }

    #[inline]
    pub fn sort_by<F>(&mut self, mut compare: F)
        where F: FnMut(&T, &T) -> Ordering,
    {
        self.items[0..self.index].sort_by(|a, b| unsafe {
            compare(a.assume_init_ref(), b.assume_init_ref())
        })
    }

    #[inline]
    pub fn sort_unstable_by<F>(&mut self, mut compare: F)
        where F: FnMut(&T, &T) -> Ordering,
    {
        self.items[0..self.index].sort_unstable_by(|a, b| unsafe {
            compare(a.assume_init_ref(), b.assume_init_ref())
        })
    }

    #[inline]
    pub unsafe fn ptr_at(&self, index: usize) -> *const T {
        self.items[index].as_ptr()
    }

    #[inline]
    pub unsafe fn ptr_at_mut(&mut self, index: usize) -> *mut T {
        self.items[index].as_mut_ptr()
    }

    #[inline]
    pub unsafe fn read_at(&self, index: usize) -> T {
        self.items[index].assume_init_read()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.index = len;
    }

    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.items[0..self.index]
            .iter()
            .map(|x| unsafe { x.assume_init_ref() })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> {
        self.items[0..self.index]
            .iter_mut()
            .map(|x| unsafe { x.assume_init_mut() })
    }
}

impl<T: Default, const N: usize> ConstVec<T, N> {
    pub fn init_default(&mut self) {
        for i in 0..N {
            self.items[i].write(T::default());
        }

        self.index = self.capacity();
    }
}

impl<T, const N: usize> Index<usize> for ConstVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.index, "Index out of bounds.");

        unsafe {
            self.items[index].assume_init_ref()
        }
    }
}

impl<T, const N: usize> IndexMut<usize> for ConstVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.index, "Index out of bounds.");

        unsafe {
            self.items[index].assume_init_mut()
        }
    }
}

impl<T, const N: usize> AsRef<[T]> for ConstVec<T, N> {
    fn as_ref(&self) -> &[T] {
        // SAFETY: &[T] and &[MaybeUninit<T>] have the same layout
        unsafe { mem::transmute(&self.items[0..self.index]) }
    }
}

impl<T, const N: usize> Drop for ConstVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: Clone + Copy, const N: usize> Clone for ConstVec<T, N> {
    fn clone(&self) -> Self {
        Self { index: self.index, items: self.items.clone() }
    }
}

impl<T: Default, const N: usize> Default for ConstVec<T, N> {
    fn default() -> Self {
        let mut instance = Self::new();
        instance.init_default();

        instance
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for ConstVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_ref()).finish()
    }
}
