use std::collections::VecDeque;

pub struct BufferedPeekable<I: Iterator> {
    iter: I,
    buffer: VecDeque<I::Item>,
    buf_size: usize,
}

impl<I: Iterator> BufferedPeekable<I> {
    pub fn new(iter: I, buf_size: usize) -> Self {
        BufferedPeekable {
            iter,
            buffer: VecDeque::new(),
            buf_size,
        }
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_at(0)
    }

    pub fn peek_at(&mut self, idx: usize) -> Option<&I::Item> {
        if idx >= self.buf_size {
            return None;
        }

        while self.buffer.len() <= idx {
            if let Some(item) = self.iter.next() {
                self.buffer.push_back(item);
            } else {
                break;
            }
        }

        self.buffer.get(idx)
    }
}

impl<I> Iterator for BufferedPeekable<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            self.iter.next()
        } else {
            self.buffer.pop_front()
        }
    }
}
