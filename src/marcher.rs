use std::fmt::Debug;
use std::ops::Range;

pub struct Marcher<T> {
    values: Vec<T>,
    pub curr: i32,
}

#[allow(dead_code)]
impl<T> Marcher<T>
where
    T: PartialEq + Debug,
{
    pub fn new(values: Vec<T>) -> Self {
        Marcher { values, curr: -1 }
    }

    pub fn advance(&mut self, offset: usize) -> Option<&T> {
        let t = self.values.get((self.curr + offset as i32) as usize);

        if t.is_some() {
            self.curr += offset as i32
        }
        t
    }

    pub fn peek_bak(&self, offset: usize) -> Option<&T> {
        if self.curr < 0 {
            return None;
        }
        self.values.get((self.curr - offset as i32) as usize)
    }

    pub fn peek_for(&self, offset: usize) -> Option<&T> {
        self.values.get((self.curr + offset as i32) as usize)
    }

    pub fn peek_range(&self, range: Range<usize>) -> Option<&[T]> {
        self.values.get(range)
    }

    pub fn advance_until(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<&[T]> {
        let start = self.curr;
        loop {
            match self.peek_for(1) {
                Some(t) => match predicate(t) {
                    true => break,
                    false => self.advance(1),
                },
                None => {
                    break;
                }
            };
        }
        self.peek_range(start as usize..self.curr as usize)
    }

    pub fn advance_if(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<&T> {
        match self.peek_for(1) {
            Some(t) => {
                if predicate(t) {
                    return self.advance(1);
                }
                return None;
            }
            None => return None,
        };
    }

    pub fn completed(&self) -> bool {
        self.values.len() == (self.curr + 1) as usize
    }

    pub fn reset(&mut self, values: Vec<T>) {
        self.values = values;
        self.curr = 0;
    }
}

impl<T> Default for Marcher<T> {
    fn default() -> Self {
        Marcher {
            values: Vec::<T>::new(),
            curr: 0,
        }
    }
}
