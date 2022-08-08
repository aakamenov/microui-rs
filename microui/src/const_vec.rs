use std::{
    mem::MaybeUninit,
    cmp::Ordering,
    ops::{Index, IndexMut},
    ptr,
    any
};

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

    #[inline]
    pub fn push(&mut self, item: T) {
        assert!(
            self.index < self.items.len(),
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
                Some(ptr::read(self.items[self.index].as_ptr()))
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
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
    pub fn sort<F>(&mut self, mut compare: F)
        where F: FnMut(&T, &T) -> Ordering,
    {
        self.items[0..self.index].sort_unstable_by(|a, b| unsafe {
            compare(a.assume_init_ref(), b.assume_init_ref())
        })
    }

    #[inline]
    pub unsafe fn ptr_at(&self, index: usize) -> *const T {
        assert!(index < self.items.len());

        self.items[index].as_ptr()
    }

    #[inline]
    pub unsafe fn ptr_at_mut(&mut self, index: usize) -> *mut T {
        assert!(index < self.items.len());

        self.items[index].as_mut_ptr()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items[0..self.index]
            .iter()
            .map(|x| unsafe { x.assume_init_ref() })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items[0..self.index]
            .iter_mut()
            .map(|x| unsafe { x.assume_init_mut() })
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
