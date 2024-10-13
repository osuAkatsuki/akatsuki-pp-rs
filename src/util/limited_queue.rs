use std::{mem, ops::Index, slice::Iter};

/// Efficient counterpart to osu!'s [`LimitedCapacityQueue`] i.e. an indexed
/// queue with limited capacity.
///
/// [`LimitedQueue`] will use an internal array as queue which is stored on
/// the stack. Hence, if `size_of<T>() * N` is very large, consider using a
/// different type since heap allocation might be favorable.
///
/// [`LimitedCapacityQueue`]: https://github.com/ppy/osu/blob/b49a1aab8ac6e16e48dffd03f55635cdc1771adf/osu.Game/Rulesets/Difficulty/Utils/LimitedCapacityQueue.cs
#[derive(Clone, Debug)]
pub struct LimitedQueue<T, const N: usize> {
    queue: [T; N],
    /// If the queue is not empty, `end` is the index of the last element.
    /// Otherwise, it has no meaning.
    end: usize,
    /// Amount of elements in the queue. This is equal to `end + 1` if the
    /// queue is not full, or `N` otherwise.
    len: usize,
}

impl<T, const N: usize> Default for LimitedQueue<T, N>
where
    T: Copy + Clone + Default,
{
    fn default() -> Self {
        Self {
            end: N - 1,
            queue: [T::default(); N],
            len: 0,
        }
    }
}

impl<T, const N: usize> LimitedQueue<T, N>
where
    T: Copy + Clone + Default,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, const N: usize> LimitedQueue<T, N> {
    pub fn push(&mut self, elem: T) {
        self.end = (self.end + 1) % N;
        self.queue[self.end] = elem;
        self.len += usize::from(self.len < N);
    }

    #[cfg(test)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    #[cfg(test)]
    pub const fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.queue[self.end])
        }
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.is_full() {
            (&self.queue[self.end + 1..N], &self.queue[0..=self.end])
        } else {
            (&[], &self.queue[0..self.len])
        }
    }

    pub fn iter(&self) -> LimitedQueueIter<'_, T> {
        let (head, tail) = self.as_slices();

        LimitedQueueIter {
            head: head.iter(),
            tail: tail.iter(),
        }
    }
}

pub struct LimitedQueueIter<'a, T> {
    head: Iter<'a, T>,
    tail: Iter<'a, T>,
}

impl<'a, T> Iterator for LimitedQueueIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let val @ Some(_) = self.head.next() {
            val
        } else {
            mem::swap(&mut self.head, &mut self.tail);

            self.head.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();

        (len, Some(len))
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn nth(&mut self, mut n: usize) -> Option<Self::Item> {
        if self.head.len() <= n {
            n -= self.head.len();
            mem::swap(&mut self.head, &mut self.tail);
            self.tail = [].iter();
        }

        self.head.nth(n)
    }
}

impl<T> DoubleEndedIterator for LimitedQueueIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let val @ Some(_) = self.tail.next_back() {
            val
        } else {
            mem::swap(&mut self.head, &mut self.tail);

            self.tail.next_back()
        }
    }
}

impl<T> ExactSizeIterator for LimitedQueueIter<'_, T> {
    fn len(&self) -> usize {
        self.head.len() + self.tail.len()
    }
}

impl<T, const N: usize> Index<usize> for LimitedQueue<T, N> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        let idx = (idx + usize::from(self.len == N) * (self.end + 1)) % N;

        &self.queue[idx]
    }
}
