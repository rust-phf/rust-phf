use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct ArrayVec<T: Copy, const N: usize> {
    arr: [T; N],
    len: usize,
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
    #[inline(always)]
    pub const fn new(marker: T) -> Self {
        Self {
            arr: [marker; N],
            len: 0,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len + 1
    }

    #[inline]
    pub const fn push(&mut self, value: T) {
        self.arr[self.len] = value;
        self.len += 1;
    }

    #[inline]
    pub const fn pop(&mut self) -> T {
        self.len -= 1;
        self.arr[self.len]
    }

    #[inline]
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub const fn get(&self, i: usize) -> T {
        assert!(i < self.len);
        self.arr[i]
    }

    #[inline]
    pub const fn get_ref(&self, i: usize) -> &T {
        assert!(i < self.len);
        &self.arr[i]
    }

    #[inline]
    pub const fn set(&mut self, i: usize, value: T) {
        assert!(i <= self.len);
        self.arr[i] = value;
    }
}

impl<T: Copy, const N: usize> const Deref for ArrayVec<T, N> {
    type Target = [T; N];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.arr
    }
}
