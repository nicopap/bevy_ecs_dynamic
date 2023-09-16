use std::{collections::HashSet, fmt, iter};

use bevy_ecs::{component::ComponentId, world::unsafe_world_cell::UnsafeEntityCell};
use bevy_reflect::ReflectFromPtr;
use fixedbitset::FixedBitSet;
use tracing::trace;

use crate::builder::{Fetch, FetchData};
use crate::debug_unchecked::DebugUnchecked;
use crate::dynamic_query::DynamicItem;
use crate::jagged_array::{JaggedArray, JaggedArrayBuilder};

#[derive(Clone)]
pub struct FetchComponent {
    id: ComponentId,
    from_ptr: ReflectFromPtr,
}
impl fmt::Debug for FetchComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Fetch").field(&self.id).finish()
    }
}

#[derive(Clone, Debug)]
pub struct Fetches {
    pub(crate) has_entity: bool,
    // TODO(perf): do not store the TypeId, which is 128 bits
    pub(crate) components: JaggedArray<FetchComponent>,
}
impl Fetches {
    pub fn new(mut fetches: Vec<Fetch>) -> Option<Self> {
        fetches.sort_unstable();
        let has_entity = fetches.last() == Some(&Fetch::Entity);
        if has_entity {
            trace!("Fetch has entity");
            fetches.pop();
        }
        let mut builder = JaggedArrayBuilder::new_with_capacity(4, fetches.len());
        let mut last_idx = 0;
        for fetch in fetches.into_iter() {
            let index = fetch.discriminant_index();
            for i in last_idx..index {
                trace!("^^^ Fetch row {i} ^^^");
                builder.add_row(iter::empty());
            }
            last_idx = index;

            let FetchData { id, from_ptr } = fetch.data().clone();
            trace!("Fetch has component ID {id:?}");
            builder.add_elem(FetchComponent { id, from_ptr });
        }
        for i in last_idx..4 {
            trace!("^^ Fetch row {i} ^^");
            builder.add_row(iter::empty());
        }
        let components = builder.build();

        // SAFETY:
        // - last_idx = index = fetch.discriminant_index()
        // - fetch.discriminant_index() ∈ {0,1,2,3,4}
        // - We panic when fetch.discriminant_index() = 4 because of fetch.info()
        unsafe { assert_invariant!(components.height() == 4) };

        // TODO(err): proper error reporting
        if duplicates_in(components.rows(..)) {
            return None;
        }
        Some(Fetches { has_entity, components })
    }
    pub const fn len(&self) -> usize {
        self.components.len() + (self.has_entity as u8 as usize)
    }
    #[inline]
    pub fn all_included(&self, ids: impl Iterator<Item = ComponentId>) -> bool {
        let comps = self.components.rows(Fetch::READ_IDX..=Fetch::MUT_IDX);

        let mut found = FixedBitSet::with_capacity(comps.len());

        // TODO(perf): Likely can avoid O(n²). If only `ComponedId`s were
        // ordered in `Archetype::components()`…
        for id in ids {
            if let Some(idx) = comps.iter().position(|x| x.id == id) {
                trace!("all_included: found {id:?} in fetches");
                found.set(idx, true);
            }
        }
        found.count_ones(..) == comps.len()
    }

    /// # Safety
    /// - `table` must contains the non-option components of this [`Fetches`].
    /// - You must have mut/read access to the mut/read components in this `Fetches`.
    pub unsafe fn iter<'w, 's>(&'s self, entity: UnsafeEntityCell<'w>) -> FetchesIter<'w, 's> {
        FetchesIter {
            has_entity: self.has_entity,
            fetches: &self.components,
            entity,
            row_index: 0,
            current_row: &[],
        }
    }
    /// # Safety
    /// - `table` must contains the non-option components of this [`Fetches`].
    /// - You must have read access to the mut/read components in this `Fetches`.
    pub unsafe fn iter_read_only<'w, 's>(
        &'s self,
        entity: UnsafeEntityCell<'w>,
    ) -> RoFetchesIter<'w, 's> {
        RoFetchesIter(FetchesIter {
            has_entity: self.has_entity,
            fetches: &self.components,
            entity,
            row_index: 0,
            current_row: &[],
        })
    }
}

fn duplicates_in(fetches: &[FetchComponent]) -> bool {
    let mut encountered = HashSet::with_capacity(fetches.len());
    fetches.iter().any(|fetch| !encountered.insert(fetch.id))
}

pub struct RoFetchesIter<'w, 's>(FetchesIter<'w, 's>);
pub struct FetchesIter<'w, 's> {
    has_entity: bool,
    fetches: &'s JaggedArray<FetchComponent>,
    entity: UnsafeEntityCell<'w>,
    row_index: usize,
    current_row: &'s [FetchComponent],
}
impl<'w, 's> Iterator for FetchesIter<'w, 's> {
    type Item = DynamicItem<'w>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_entity {
            self.has_entity = false;
            return Some(DynamicItem::Entity(self.entity.id()));
        }
        let Some((comp, remaining)) = self.current_row.split_first() else {
            self.current_row = self.fetches.get_row(self.row_index)?;
            self.row_index += 1;
            return self.next();
        };
        self.current_row = remaining;

        match self.row_index - 1 {
            Fetch::READ_IDX => {
                // SAFETY:
                // - (1): `Self::new`'s invariant ensures this is always Some.
                // - (2): By construction, the `ReflectFromPtr` is always the one for what we
                //   are fetching
                let ptr = unsafe { self.entity.get_by_id(comp.id).prod_unchecked_unwrap() };
                let reflect = unsafe { comp.from_ptr.as_reflect_ptr(ptr) };

                Some(DynamicItem::Read(reflect))
            }
            Fetch::MUT_IDX => {
                // SAFETY: Same as above
                let ptr = unsafe { self.entity.get_by_id(comp.id).prod_unchecked_unwrap() };
                let ptr = unsafe { ptr.assert_unique() };
                let reflect = unsafe { comp.from_ptr.as_reflect_ptr_mut(ptr) };

                Some(DynamicItem::Mut(reflect))
            }
            Fetch::OPTION_READ_IDX => {
                // SAFETY: Same as point (3) of above
                let ptr = unsafe { self.entity.get_by_id(comp.id) };
                let reflect = unsafe { ptr.map(|p| comp.from_ptr.as_reflect_ptr(p)) };

                Some(DynamicItem::OptionRead(reflect))
            }
            Fetch::OPTION_MUT_IDX => {
                // SAFETY: Same as point (3) of above
                let ptr = unsafe { self.entity.get_by_id(comp.id) };
                let ptr = unsafe { ptr.map(|p| p.assert_unique()) };
                let reflect = unsafe { ptr.map(|p| comp.from_ptr.as_reflect_ptr_mut(p)) };

                Some(DynamicItem::OptionMut(reflect))
            }
            // TODO(perf): check this is elided
            _ => unreachable!(),
        }
    }
}

impl<'w, 's> Iterator for RoFetchesIter<'w, 's> {
    type Item = DynamicItem<'w>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.has_entity {
            self.0.has_entity = false;
            return Some(DynamicItem::Entity(self.0.entity.id()));
        }
        let Some((comp, remaining)) = self.0.current_row.split_first() else {
            self.0.current_row = self.0.fetches.get_row(self.0.row_index)?;
            self.0.row_index += 1;
            return self.0.next();
        };
        self.0.current_row = remaining;

        match self.0.row_index - 1 {
            Fetch::MUT_IDX | Fetch::READ_IDX => {
                // SAFETY:
                // - (1): `Self::new`'s invariant ensures this is always Some.
                // - (2): By construction, the `ReflectFromPtr` is always the one for what we
                //   are fetching
                let ptr = unsafe { self.0.entity.get_by_id(comp.id).prod_unchecked_unwrap() };
                let reflect = unsafe { comp.from_ptr.as_reflect_ptr(ptr) };

                Some(DynamicItem::Read(reflect))
            }
            Fetch::OPTION_MUT_IDX | Fetch::OPTION_READ_IDX => {
                // SAFETY: Same as point (3) of above
                let ptr = unsafe { self.0.entity.get_by_id(comp.id) };
                let reflect = unsafe { ptr.map(|p| comp.from_ptr.as_reflect_ptr(p)) };

                Some(DynamicItem::OptionRead(reflect))
            }
            _ => {
                // SAFETY: The `fetches` iterator comes from `Fetches.components`,
                // which is built in `Fetches::new`, which builds a JaggedArray with
                // at most 4 rows due to reasons evoked in the assert_invariant!
                // in `Fetches::new`
                unsafe { assert_invariant!(false) };
                None
            }
        }
    }
}
