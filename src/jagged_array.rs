//! A variable length matrix optimized for read-only rows.

use std::ops::RangeBounds;

/// A variable length matrix optimized for read-only rows.
#[derive(Debug, PartialEq, Eq, Clone)]
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
    /// Same as `row`, but for a range of rows instead of individual rows.
    pub fn rows(&self, range: impl RangeBounds<usize>) -> &[V] {
        let start = match range.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(start) => *start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.height(),
        };
        assert!(end <= self.ends.len() + 1);
        assert!(start <= end);

        let get_end = |end: &u32| *end as usize;
        let start = start.checked_sub(1).map_or(0, |i| self.ends[i]) as usize;
        let end = self.ends.get(end).map_or(self.data.len(), get_end);
        &self.data[start..end]
    }
    pub fn rows_iter(&self) -> JaggedArrayRows<V> {
        JaggedArrayRows { array: self, row: 0 }
    }
}

pub struct JaggedArrayRows<'j, V> {
    array: &'j JaggedArray<V>,
    row: usize,
}
impl<'j, V> Iterator for JaggedArrayRows<'j, V> {
    type Item = &'j [V];

    fn next(&mut self) -> Option<Self::Item> {
        if self.row >= self.array.ends.len() {
            return None;
        }
        let get_end = |end: &u32| *end as usize;
        let ends = &self.array.ends;
        let start = self.row.checked_sub(1).map_or(0, |i| ends[i]) as usize;
        let end = ends.get(self.row).map_or(self.array.data.len(), get_end);
        self.array.data.get(start..end)
    }
}
pub struct JaggedArrayBuilder<V> {
    last_end: Option<u32>,
    ends: Vec<u32>,
    data: Vec<V>,
}
impl<V> JaggedArrayBuilder<V> {
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
    pub fn add_row(&mut self, row: impl IntoIterator<Item = V>) {
        self.data.extend(row);
        if let Some(last_end) = self.last_end.replace(self.data.len() as u32) {
            self.ends.push(last_end);
        }
    }
    pub fn build(self) -> JaggedArray<V> {
        JaggedArray { ends: self.ends.into(), data: self.data.into() }
    }
}
