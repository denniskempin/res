use std::collections::vec_deque::Iter;
use std::collections::VecDeque;

use bincode::Decode;
use bincode::Encode;

#[derive(Default, Encode, Decode, Clone)]
pub struct RingBuffer<T, const N: usize> {
    stack: VecDeque<T>,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    pub fn pop(&mut self) -> T {
        self.stack.pop_front().unwrap()
    }

    pub fn push(&mut self, system: T) {
        self.stack.push_front(system);
        self.stack.truncate(N);
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.stack.iter()
    }
}
