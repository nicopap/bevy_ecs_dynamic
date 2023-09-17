use bevy_ecs::archetype::Archetypes;
use bevy_ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell};
use bevy_ecs::{component::Tick, prelude::Entity, world::World};
use thiserror::Error;

use crate::archematch::MatchedArchetypes;
use crate::dynamic_query::{DynamicItem, DynamicQuery};
use crate::iter::{DynamicQueryIter, RoDynamicQueryIter};
use crate::maybe_item::{assume_init_mut, MaybeDynamicItem};
use crate::{fetches::Fetches, filters::Filters};

#[derive(Clone, Copy, Debug)]
pub struct Ticks {
    pub last_run: Tick,
    pub this_run: Tick,
}
impl Default for Ticks {
    fn default() -> Self {
        Ticks { last_run: Tick::new(0), this_run: Tick::new(0) }
    }
}
impl Ticks {
    pub fn new(last_run: Tick, this_run: Tick) -> Self {
        Self { last_run, this_run }
    }
}

#[derive(Debug, Error)]
pub enum DynamicQueryError {
    #[error("No entity with id {0:?} exists in the world.")]
    Dangling(Entity),
    #[error(
        "Entity with id {0:?} doesn't have the right set of components to satisfy DynamicState."
    )]
    Unmatched(Entity),
}

#[derive(Clone, Debug)]
pub struct DynamicState {
    pub(crate) fetches: Fetches,
    pub(crate) filters: Filters,
    pub(crate) archetype_ids: MatchedArchetypes,
    item_buffer: Box<[MaybeDynamicItem]>,
}
impl DynamicState {
    // TODO(feat): Query caching
    pub fn in_world(query: &DynamicQuery, world: &mut World) -> Self {
        Self::new(query, world.archetypes())
    }
    pub fn new(query: &DynamicQuery, world: &Archetypes) -> Self {
        let item_count = query.fetches.len();
        let item_buffer = vec![MaybeDynamicItem::uninit(); item_count].into();

        let mut state = DynamicState {
            fetches: query.fetches.clone(),
            filters: query.filters.clone(),
            archetype_ids: MatchedArchetypes::new(&query.fetches, &query.filters, world),
            item_buffer,
        };
        state
    }
    /// Verify if this `DynamicState` matches `archetype`, adding it to its internal list
    /// of archetypes and returns `true` if so.
    ///
    /// This returns `true` even if the archetype is already part of the list.
    ///
    /// # Performance
    ///
    /// This is `O(n * c)` where 'n' is the size of the archetype and 'c' is
    /// the number of filter conjunctions (ie: `Or` clauses).
    pub fn add_archetypes(&mut self, archetypes: &Archetypes) {
        self.archetype_ids
            .add_archetypes(&self.fetches, &self.filters, archetypes);
    }

    /// Overwrites `self.item_buffer` with the `fetch` items from provided
    /// table row and returns the buffer as-is.
    fn buffer_row<'s, 'w>(&'s mut self, entity: UnsafeEntityCell<'w>) -> &'s mut [DynamicItem<'w>] {
        // SAFETY: by construction item_buffer is same length as self.fetches
        unsafe { assert_invariant!(self.fetches.len() == self.item_buffer.len()) };

        // We know fetches.len() equals self.item_buffer, because we used that
        // value to create item_buffer
        let iter = unsafe { self.fetches.iter(entity) };
        self.item_buffer.iter_mut().zip(iter).for_each(|(i, v)| {
            i.set(v);
        });
        // SAFETY: we just initialized all buffer items
        unsafe { assume_init_mut(self.item_buffer.as_mut()) }
    }

    pub fn get_unchecked_manual<'w, 's>(
        &'s mut self,
        world: UnsafeWorldCell<'w>,
        entity: Entity,
    ) -> Result<&'s mut [DynamicItem<'w>], DynamicQueryError> {
        let ticks = Ticks {
            last_run: world.last_change_tick(),
            this_run: world.change_tick(),
        };
        let dangling_entity = DynamicQueryError::Dangling(entity);
        let entity = world.get_entity(entity).ok_or(dangling_entity)?;
        let archetype = entity.archetype();
        let getter = self.archetype_ids.getter(&self.filters);
        if !getter.contains(ticks, entity) {
            return Err(DynamicQueryError::Unmatched(entity.id()));
        }
        Ok(self.buffer_row(entity))
    }
    pub fn get<'w, 's>(
        &'s mut self,
        world: &'w World,
        entity: Entity,
    ) -> Result<&'s [DynamicItem<'w>], DynamicQueryError> {
        let world = world.as_unsafe_world_cell_readonly();
        self.get_unchecked_manual(world, entity).map(|x| &*x)
    }
    pub fn get_mut<'w, 's>(
        &'s mut self,
        world: &'w mut World,
        entity: Entity,
    ) -> Result<&'s mut [DynamicItem<'w>], DynamicQueryError> {
        let world = world.as_unsafe_world_cell();
        self.get_unchecked_manual(world, entity)
    }
    pub fn iter<'w, 's>(&'s mut self, world: &'w World) -> RoDynamicQueryIter<'w, 's> {
        let world = world.as_unsafe_world_cell_readonly();
        RoDynamicQueryIter::new(world, self)
    }
    pub fn iter_mut<'w, 's>(&'s mut self, world: &'w mut World) -> DynamicQueryIter<'w, 's> {
        let world = world.as_unsafe_world_cell();
        DynamicQueryIter::new(world, self)
    }
}
