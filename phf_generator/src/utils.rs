use core::ops::Deref;

#[derive(Clone, Copy)]
pub struct ArrayVec<T: Copy, const N: usize> {
    arr: [T; N],
    len: usize,
}

impl<T: Copy, const N: usize> ArrayVec<T, N> {
    #[inline(always)]
    pub const fn new_empty(marker: T) -> Self {
        Self {
            arr: [marker; N],
            len: 0,
        }
    }

    #[inline(always)]
    pub const fn new_full(marker: T) -> Self {
        Self {
            arr: [marker; N],
            len: N,
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.arr.len()
    }

    #[inline]
    pub const fn push(&mut self, value: T) {
        assert!(self.len() < self.capacity());
        self.arr[self.len] = value;
        self.len += 1;
    }

    #[inline]
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub const fn get(&self, i: usize) -> T {
        assert!(i < self.len());
        self.arr[i]
    }

    #[inline]
    pub const fn get_ref(&self, i: usize) -> &T {
        assert!(i < self.len());
        &self.arr[i]
    }

    #[inline]
    pub const fn set(&mut self, i: usize, value: T) {
        if i == self.len() {
            self.push(value);
        } else {
            assert!(i < self.len());
            self.arr[i] = value;
        }
    }
}

impl<T: Copy, const N: usize> const Deref for ArrayVec<T, N> {
    type Target = [T; N];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.arr
    }
}

#[cfg(test)]
mod tests {
    use super::ArrayVec;

    #[test]
    fn test_api() {
        let mut arr = ArrayVec::<usize, 10>::new_empty(0);
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.capacity(), 10);

        arr.push(1);
        arr.push(2);
        arr.push(4);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.capacity(), 10);
        assert_eq!(arr.get(2), 4);
        assert_eq!(arr.get(0), 1);

        arr.push(4);
        arr.set(2, 3);
        assert_eq!(arr.get(2), 3);
        assert_eq!(arr.get(arr.len() - 1), 4);

        arr.clear();
        assert_eq!(arr.len(), 0);
    }
}
