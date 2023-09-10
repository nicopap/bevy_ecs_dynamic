use std::{collections::HashSet, fmt, iter, slice};

use bevy_ecs::component::{ComponentId, ComponentInfo};
use bevy_ecs::world::unsafe_world_cell::UnsafeEntityCell;
use bevy_reflect::{ReflectFromPtr, TypeRegistry};
use fixedbitset::FixedBitSet;

use crate::ctor_dsl::Fetch;
use crate::debug_unchecked::DebugUnchecked;
use crate::dynamic_query::DynamicItem;
use crate::jagged_array::{JaggedArray, JaggedArrayBuilder, JaggedArrayRows};

#[derive(Clone)]
pub struct FetchComponent {
    id: ComponentId,
    from_ptr: ReflectFromPtr,
}
impl fmt::Debug for FetchComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FetchComponent")
            .field("id", &self.id)
            .field("from_ptr", &"<inscrutable ReflectFromPtr>")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Fetches {
    pub(crate) has_entity: bool,
    // TODO(perf): do not store the TypeId, which is 128 bits
    pub(crate) components: JaggedArray<FetchComponent>,
}
impl Fetches {
    pub fn new(mut fetches: Vec<Fetch>, registry: &TypeRegistry) -> Option<Self> {
        let get_registration = |info: &ComponentInfo| {
            info.type_id().map_or_else(
                || registry.get_with_name(info.name()),
                |id| registry.get(id),
            )
        };
        fetches.sort_unstable();
        let has_entity = fetches.last() == Some(&Fetch::Entity);
        if has_entity {
            fetches.pop();
        }
        let mut builder = JaggedArrayBuilder::new_with_capacity(4, fetches.len());
        let mut last_idx = 0;
        for fetch in fetches.into_iter() {
            let index = fetch.discriminant_index();
            if last_idx != index {
                builder.add_row(iter::empty());
            }
            last_idx = index;

            let info = fetch.info();
            let registration = get_registration(info)?;
            let from_ptr = registration.data::<ReflectFromPtr>()?.clone();
            builder.add_elem(FetchComponent { id: info.id(), from_ptr });
        }
        let components = builder.build();

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
                found.set(idx, true);
            }
        }
        found.count_ones(..) == comps.len()
    }

    /// # Safety
    /// - `table` must contains the non-option components of this [`Fetches`].
    /// - You must have mut/read access to the mut/read components in this `Fetches`.
    pub unsafe fn iter<'s, 'w>(
        &'s self,
        entity: UnsafeEntityCell<'w>,
    ) -> FetchesIter<'s, 'w, impl FnMut(FetchComponent) -> (usize, FetchComponent)> {
        unsafe { FetchesIter::new(self.has_entity, &self.components, entity) }
    }
}

fn duplicates_in(fetches: &[FetchComponent]) -> bool {
    let mut encountered = HashSet::with_capacity(fetches.len());
    fetches.iter().any(|fetch| !encountered.insert(fetch.id))
}

// TODO(clean): eyebleed
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
    entity: UnsafeEntityCell<'w>,
}
impl<'s, 'w> FetchesIter<'s, 'w, fn(FetchComponent) -> (usize, FetchComponent)> {
    /// # Safety
    /// - `table` must contains the non-option components of this [`Fetches`].
    /// - You must have mut/read access to the mut/read components in this `Fetches`.
    unsafe fn new(
        has_entity: bool,
        fetches: &'s JaggedArray<FetchComponent>,
        entity: UnsafeEntityCell<'w>,
    ) -> FetchesIter<'s, 'w, impl FnMut(FetchComponent) -> (usize, FetchComponent)> {
        FetchesIter {
            has_entity,
            fetches: fetches.rows_iter().enumerate().flat_map(mapmap),
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
            return Some(DynamicItem::Entity(self.entity.id()));
        }
        let (i, comp) = self.fetches.next()?;
        match i {
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
