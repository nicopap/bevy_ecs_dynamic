use std::iter;

use bevy_ecs::archetype::{Archetype, ArchetypeId, Archetypes};
use bevy_ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell};
use bevy_ecs::{component::Tick, prelude::Entity, world::World};
use fixedbitset::{FixedBitSet, Ones};
use thiserror::Error;
use tracing::trace;

use crate::dynamic_query::{DynamicItem, DynamicQuery};
use crate::iter::{DynamicQueryIter, RoDynamicQueryIter};
use crate::maybe_item::{assume_init_mut, MaybeDynamicItem};
use crate::{fetches::Fetches, filters::Filters};

fn usize_to_archetype_id(usize: usize) -> ArchetypeId {
    let u32 = usize as u32;
    unsafe { std::mem::transmute(u32) }
}
fn archetype_id_to_u32(id: ArchetypeId) -> u32 {
    // SAFETY: ArchetypeId is repr(transparent) u32
    unsafe { std::mem::transmute(id) }
}
#[derive(Default, Clone, Debug)]
pub(crate) struct MatchedArchetypes(FixedBitSet);

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

impl MatchedArchetypes {
    fn add_archetype(&mut self, id: ArchetypeId) {
        let id = archetype_id_to_u32(id);
        self.0.grow(id as usize + 1);
        self.0.set(id as usize, true);
    }

    fn contains(&self, entity_archetype: ArchetypeId) -> bool {
        trace!("Contains {entity_archetype:?}?");
        let id = archetype_id_to_u32(entity_archetype);
        let yes = self.0.contains(id as usize);
        trace!("> {yes}");
        yes
    }

    pub(crate) fn iter(&self) -> iter::Map<Ones, fn(usize) -> ArchetypeId> {
        self.0.ones().map(usize_to_archetype_id)
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
    #[error("A `Changed` or `Added` filter means {0:?} doesn't satisfy DynamicState.")]
    NotInTick(Entity),
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
            archetype_ids: MatchedArchetypes::default(),
            item_buffer,
        };
        for archetype in world.iter() {
            state.add_archetype(archetype);
        }
        state
    }
    // TODO(perf): O(n * c) where 'n' size of archetype & 'c' number of conjunctions
    /// Verify if this `DynamicState` matches `archetype`, adding it to its internal list
    /// of archetypes and returns `true` if so.
    ///
    /// This returns `true` even if the archetype is already part of the list.
    ///
    /// # Performance
    ///
    /// This is `O(n * c)` where 'n' is the size of the archetype and 'c' is
    /// the number of filter conjunctions (ie: `Or` clauses).
    pub fn add_archetype(&mut self, archetype: &Archetype) -> bool {
        let conjs = self.filters.len();
        trace!("add_arch? {:?}, {conjs} conjs", archetype.id());
        let matches = if self.filters.is_empty() {
            self.fetches.all_included(archetype.components())
        } else {
            let mut conjunctions = self.filters.conjunctions();
            conjunctions.any(|c| c.includes(&self.fetches, archetype))
        };
        if matches {
            self.archetype_ids.add_archetype(archetype.id());
            trace!("> matches!");
        }
        matches
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
        if !self.archetype_ids.contains(archetype.id()) {
            return Err(DynamicQueryError::Unmatched(entity.id()));
        }

        let mut conjunctions = self.filters.conjunctions();
        if !conjunctions.any(|c| c.within_tick(ticks, entity)) {
            return Err(DynamicQueryError::NotInTick(entity.id()));
        }
        drop(conjunctions);
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
