//! A variable length matrix optimized for read-only
//! rows and statically known row count.

use std::{mem::size_of, ops::RangeBounds};

/// A variable length matrix optimized for read-only rows.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JaggedArray<V> {
    // TODO(perf): store the row indices inline, preventing cache misses when looking up several rows.
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
    /// Create a [`JaggedArray`] of ` + 1` rows, values of `ends` are the
    /// end indicies (exclusive) of each row in `data`.
    pub fn new(ends: Box<[u32]>, data: Box<[V]>) -> Option<Self> {
        assert!(size_of::<usize>() >= size_of::<u32>());

        let mut previous_end = 0;
        let last_end = data.len() as u32;
        for (i, end) in ends.iter().enumerate() {
            if *end > last_end {
                return None;
            }
            if *end < previous_end {
                return None;
            }
            previous_end = *end;
        }
        Some(Self { ends, data })
    }
    /// Get slice to row at given `index`.
    #[inline]
    pub fn row(&self, index: usize) -> &[V] {
        assert!(index <= self.ends.len());
        // TODO(perf): verify generated code elides bound checks.
        let get_end = |end: &u32| *end as usize;

        let start = index.checked_sub(1).map_or(0, |i| self.ends[i]) as usize;
        let end = self.ends.get(index).map_or(self.data.len(), get_end);
        &self.data[start..end]
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
    /// Get `V` at exact `direct_index` ignoring row sizes,
    /// acts as if the whole array was a single row.
    #[inline]
    pub fn get(&self, direct_index: usize) -> Option<&V> {
        self.data.get(direct_index)
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
