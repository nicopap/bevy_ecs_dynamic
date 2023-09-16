//! A variable length matrix optimized for read-only rows.

use core::fmt;
use std::ops::{
    Bound::{Excluded, Included},
    RangeBounds,
};

/// A variable length matrix optimized for read-only rows.
#[derive(PartialEq, Eq, Clone)]
pub struct JaggedArray<V> {
    ends: Box<[u32]>,
    data: Box<[V]>,
}

impl<V> JaggedArray<V> {
    /// How many cells are contained in this `JaggedArray`.
    pub const fn len(&self) -> usize {
        self.data.len()
    }
    /// Is this array empty (no cells, may have several empty rows).
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// How many rows this `JaggedArray` has.
    pub const fn height(&self) -> usize {
        self.ends.len() + 1
    }
    pub fn row(&self, index: usize) -> &[V] {
        self.get_row(index).unwrap()
    }
    pub fn rows(&self, range: impl RangeBounds<usize>) -> &[V] {
        self.get_rows(range).unwrap()
    }
    pub fn get_row(&self, index: usize) -> Option<&[V]> {
        self.get_rows(index..index + 1)
    }
    /// Same as `row`, but for a range of rows instead of individual rows.
    pub fn get_rows(&self, range: impl RangeBounds<usize>) -> Option<&[V]> {
        let get_end = |i| match i {
            n if n == self.ends.len() => Some(self.data.len()),
            n if n >= self.ends.len() => None,
            n => Some(self.ends[n] as usize),
        };
        let start = match range.start_bound() {
            Included(0) => 0,
            Included(&start) => get_end(start - 1)?,
            Excluded(&start) => get_end(start)?,
            _ => 0,
        };
        let end = match range.end_bound() {
            Excluded(0) => 0,
            Excluded(&end) => get_end(end - 1)?,
            Included(&end) => get_end(end)?,
            _ => self.data.len(),
        };
        if start > end {
            return None;
        }
        self.data.get(start..end)
    }
    pub fn rows_iter(&self) -> JaggedArrayRows<V> {
        JaggedArrayRows { array: self, row: 0 }
    }
}
impl<V: fmt::Debug> fmt::Debug for JaggedArray<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut full_array = f.debug_list();
        for row in self.rows_iter() {
            full_array.entry(&row);
        }
        full_array.finish()
    }
}

#[derive(Debug, Clone)]
pub struct JaggedArrayRows<'j, V> {
    array: &'j JaggedArray<V>,
    row: usize,
}
impl<'j, V> Iterator for JaggedArrayRows<'j, V> {
    type Item = &'j [V];

    fn next(&mut self) -> Option<Self::Item> {
        self.row += 1;
        self.array.get_row(self.row - 1)
    }
}
pub struct JaggedArrayBuilder<V> {
    last_end: Option<u32>,
    ends: Vec<u32>,
    data: Vec<V>,
}
impl<V> JaggedArrayBuilder<V> {
    pub fn new() -> Self {
        JaggedArrayBuilder { last_end: None, ends: Vec::new(), data: Vec::new() }
    }
    pub fn new_with_capacity(row_count: usize, data_len: usize) -> Self {
        JaggedArrayBuilder {
            last_end: None,
            ends: Vec::with_capacity(row_count),
            data: Vec::with_capacity(data_len),
        }
    }
    pub fn add_elem(&mut self, elem: V) {
        self.data.push(elem);
    }
    pub fn add_row(&mut self, row: impl IntoIterator<Item = V>) -> &mut Self {
        self.data.extend(row);
        if let Some(last_end) = self.last_end.replace(self.data.len() as u32) {
            self.ends.push(last_end);
        }
        self
    }
    pub fn build(&mut self) -> JaggedArray<V> {
        let ends = std::mem::take(&mut self.ends);
        let data = std::mem::take(&mut self.data);
        JaggedArray { ends: ends.into(), data: data.into() }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_row() {
        let array = JaggedArrayBuilder::new()
            .add_row([1, 2, 3])
            .add_row([4, 5, 6])
            .add_row([])
            .add_row([7, 8, 9])
            .add_row([])
            .build();

        assert_eq!(array.get_row(0), Some(&[1, 2, 3][..]));
        assert_eq!(array.get_row(1), Some(&[4, 5, 6][..]));
        assert_eq!(array.get_row(2), Some(&[][..]));
        assert_eq!(array.get_row(3), Some(&[7, 8, 9][..]));
        assert_eq!(array.get_row(4), Some(&[][..]));
        assert_eq!(array.get_row(5), None);
    }

    #[test]
    fn test_iter_rows() {
        let array = JaggedArrayBuilder::new()
            .add_row([])
            .add_row([1, 2, 3])
            .add_row([4, 5, 6])
            .add_row([])
            .add_row([7, 8, 9])
            .add_row([])
            .build();

        let mut iter = array.rows_iter();
        assert_eq!(iter.next(), Some(&[][..]));
        assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
        assert_eq!(iter.next(), Some(&[4, 5, 6][..]));
        assert_eq!(iter.next(), Some(&[][..]));
        assert_eq!(iter.next(), Some(&[7, 8, 9][..]));
        assert_eq!(iter.next(), Some(&[][..]));
        assert_eq!(iter.next(), None);
    }
    #[test]
    fn test_get_rows() {
        let array = JaggedArrayBuilder::new()
            .add_row([])
            .add_row([1, 2, 3])
            .add_row([4, 5, 6])
            .add_row([])
            .add_row([7, 8, 9])
            .add_row([])
            .build();
        println!("{array:?}");
        assert_eq!(array.get_rows(0..1), Some(&[][..]));
        assert_eq!(array.get_rows(0..2), Some(&[1, 2, 3][..]));
        assert_eq!(array.get_rows(2..2), Some(&[][..]));
        assert_eq!(array.get_rows(2..3), Some(&[4, 5, 6][..]));
        assert_eq!(array.get_rows(2..4), Some(&[4, 5, 6][..]));
        assert_eq!(array.get_rows(2..5), Some(&[4, 5, 6, 7, 8, 9][..]));
        assert_eq!(array.get_rows(..), Some(&[1, 2, 3, 4, 5, 6, 7, 8, 9][..]));
    }
}
