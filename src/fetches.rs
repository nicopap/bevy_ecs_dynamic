use std::{fmt, iter, slice};

use bevy_ecs::{component::ComponentId, storage::Table};
use bevy_reflect::ReflectFromPtr;
use fixedbitset::FixedBitSet;

use crate::debug_unchecked::DebugUnchecked;
use crate::dynamic_query::{DynamicItem, Row};
use crate::jagged_array::{JaggedArray, JaggedArrayRows};

pub enum Fetch {
    Entity,
    Read(ComponentId),
    Mut(ComponentId),
    OptionRead(ComponentId),
    OptionMut(ComponentId),
}
impl Fetch {
    const READ_IDX: usize = 0;
    const MUT_IDX: usize = 1;
    const OPTION_READ_IDX: usize = 2;
    const OPTION_MUT_IDX: usize = 3;
}

#[derive(Clone)]
pub(super) struct FetchComponent {
    id: ComponentId,
    from_ptr: ReflectFromPtr,
}
impl fmt::Debug for FetchComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FetchComponent")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Fetches {
    pub(super) has_entity: bool,
    // TODO(perf): do not store the TypeId, which is 128 bits
    pub(super) components: JaggedArray<FetchComponent>,
}
impl Fetches {
    pub const fn len(&self) -> usize {
        self.components.len() + (self.has_entity as u8 as usize)
    }
    // TODO(BUG): Doesn't work with repeated components. We need to ensure this
    // in `Fetches` constructor.
    #[inline]
    pub fn all_included(&self, ids: impl Iterator<Item = ComponentId>) -> bool {
        let comps = self.components.rows(Fetch::READ_IDX..=Fetch::MUT_IDX);

        let mut found = FixedBitSet::with_capacity(comps.len());

        // TODO(perf): Likely can avoid O(n²). If only `ComponedId`s were
        // ordered in `Archetype::components()`…
        for id in ids {
            if let Some(idx) = comps.iter().position(|x| x.id == id) {
                found.set(idx, true);
            }
        }
        found.count_ones(..) == comps.len()
    }

    // SAFETY: `table` must contains the non-option components of this [`Fetches`].
    pub unsafe fn iter<'s, 'w>(
        &'s self,
        table: &'w Table,
        entity: Row,
    ) -> FetchesIter<'s, 'w, impl FnMut(FetchComponent) -> (usize, FetchComponent)> {
        unsafe { FetchesIter::new(self.has_entity, &self.components, table, entity) }
    }
}

type FetchRowRet<'s, F> = iter::Map<iter::Cloned<slice::Iter<'s, FetchComponent>>, F>;

type FetchRows<'s, F> = iter::FlatMap<
    iter::Enumerate<JaggedArrayRows<'s, FetchComponent>>,
    FetchRowRet<'s, F>,
    fn((usize, &[FetchComponent])) -> FetchRowRet<F>,
>;
fn mapmap(
    (i, row): (usize, &[FetchComponent]),
) -> FetchRowRet<impl FnMut(FetchComponent) -> (usize, FetchComponent)> {
    row.iter().cloned().map(move |elem| (i, elem))
}

pub struct FetchesIter<'s, 'w, F: FnMut(FetchComponent) -> (usize, FetchComponent)> {
    has_entity: bool,
    fetches: FetchRows<'s, F>,
    table: &'w Table,
    entity: Row,
}
impl<'s, 'w> FetchesIter<'s, 'w, fn(FetchComponent) -> (usize, FetchComponent)> {
    // SAFETY: `table` must contains the non-option components of `fetches`.
    unsafe fn new(
        has_entity: bool,
        fetches: &'s JaggedArray<FetchComponent>,
        table: &'w Table,
        entity: Row,
    ) -> FetchesIter<'s, 'w, impl FnMut(FetchComponent) -> (usize, FetchComponent)> {
        FetchesIter {
            has_entity,
            fetches: fetches.rows_iter().enumerate().flat_map(mapmap),
            table,
            entity,
        }
    }
}
impl<'s, 'w, F: FnMut(FetchComponent) -> (usize, FetchComponent)> Iterator
    for FetchesIter<'s, 'w, F>
{
    type Item = DynamicItem<'w>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.has_entity {
            self.has_entity = false;
            return Some(DynamicItem::Entity(self.entity.entity));
        }
        let (i, comp) = self.fetches.next()?;
        let row = self.entity.table_row;
        let column = self.table.get_column(comp.id);

        match i {
            // TODO(BUG): missing handling of sparse set components
            Fetch::READ_IDX => {
                // SAFETY: `Self::new`'s invariant ensures this is always Some.
                let column = unsafe { column.release_unchecked_unwrap() };
                let ptr = unsafe { column.get_data(row).release_unchecked_unwrap() };
                let reflect = unsafe { comp.from_ptr.as_reflect_ptr(ptr) };

                Some(DynamicItem::Read(reflect))
            }
            Fetch::MUT_IDX => {
                // SAFETY: `Self::new`'s invariant ensures this is always Some.
                let column = unsafe { column.release_unchecked_unwrap() };
                let ptr = unsafe { column.get_data_mut(row).release_unchecked_unwrap() };
                let reflect = unsafe { comp.from_ptr.as_reflect_ptr_mut(ptr) };

                Some(DynamicItem::Mut(reflect))
            }
            Fetch::OPTION_READ_IDX => {
                // SAFETY: `Self::new`'s invariant ensures this is always Some.
                let ptr = column.and_then(|c| unsafe { c.get_data(row) });
                let reflect = unsafe { ptr.map(|p| comp.from_ptr.as_reflect_ptr(p)) };

                Some(DynamicItem::OptionRead(reflect))
            }
            Fetch::OPTION_MUT_IDX => {
                // SAFETY: `Self::new`'s invariant ensures this is always Some.
                let ptr = column.and_then(|c| unsafe { c.get_data_mut(row) });
                let reflect = unsafe { ptr.map(|p| comp.from_ptr.as_reflect_ptr_mut(p)) };

                Some(DynamicItem::OptionMut(reflect))
            }
            // TODO(perf): check this is elided
            _ => unreachable!(),
        }
    }
}
