use std::collections::hash_map::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use std::ops::Range;

/// A Marcher is a container for a vector of elements that advances through
/// the vector of values one at a time until reaching the end of the vector.
///
/// It allows for conditional dispatch depending on the value of the proceding element.
///
/// The Marcher starts at usize::MAX and wraps around to the zeroth element
/// on the first call of advance to simulate starting at the begining of the array.
///
/// # Example
/// ```rust
/// ```
pub struct Marcher<T> {
    values: Vec<T>,
    pub curr: usize,
}

#[allow(dead_code)]
impl<T> Marcher<T>
where
    T: PartialEq + Debug + Clone,
{
    pub fn new(values: Vec<T>) -> Self {
        Marcher {
            values,
            curr: usize::MAX,
        }
    }

    /// Increments the position of the marcher by the prescribed offset.
    ///
    /// The Marcher starts at usize::MAX and wraps around to the zeroth element
    /// on the first call of advance to simulate starting at the begining of the array.
    ///
    /// # Example
    /// ```rust
    ///let mut m: Marcher<i32> = Marcher::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///assert!(1, m.advance(1).unwrap());
    /// ```
    pub fn advance(&mut self, offset: usize) -> Option<&T> {
        let (val, _ov) = self.curr.overflowing_add(offset);
        self.curr = val;

        let t = self.values.get(self.curr);
        t
    }

    pub fn peek(&self, offset: isize) -> Option<&T> {
        let add = offset >= 0;
        let pos = match add {
            true => {
                let (val, _) = self.curr.overflowing_add(offset as usize);
                val
            }
            false => {
                let (val, _) = self.curr.overflowing_sub(offset.abs() as usize);
                val
            }
        };
        self.values.get(pos)
    }

    pub fn peek_range(&self, range: Range<usize>) -> Option<&[T]> {
        self.values.get(range)
    }

    pub fn advance_until(
        &mut self,
        mut predicate: impl FnMut(&mut Marcher<T>, &T) -> bool,
    ) -> Option<&[T]> {
        let start = self.curr;
        while let Some(t) = self.peek(1) {
            match predicate(self, &t.clone()) {
                true => break,
                false => {
                    self.advance(1);
                }
            }
        }
        self.peek_range(start..self.curr)
    }

    pub fn advance_if(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<&T> {
        match self.peek(1) {
            Some(t) => {
                if predicate(&t.clone()) {
                    return self.advance(1);
                };
                None
            }
            None => None,
        }
    }

    pub fn completed(&self) -> bool {
        let (len, _) = self.curr.overflowing_add(1 as usize);
        self.values.len() == len
    }
}

impl<T> Default for Marcher<T> {
    fn default() -> Self {
        Marcher {
            values: Vec::<T>::new(),
            curr: usize::MAX,
        }
    }
}
