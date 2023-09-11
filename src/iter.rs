use std::iter;

use bevy_ecs::archetype::{ArchetypeEntity, ArchetypeId};
use bevy_ecs::world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell};
use fixedbitset::Ones;

use crate::debug_unchecked::DebugUnchecked;
use crate::state::Ticks;
use crate::{fetches::Fetches, filters::Filters, DynamicItem, DynamicState};

fn fetch_buffer_ro<'w>(fetches: &Fetches, entity: UnsafeEntityCell<'w>) -> Box<[DynamicItem<'w>]> {
    let mut item_buffer = Vec::with_capacity(fetches.len());
    for fetch in unsafe { fetches.iter_read_only(entity) } {
        item_buffer.push(fetch);
    }
    item_buffer.into()
}
fn fetch_buffer<'w>(fetches: &Fetches, entity: UnsafeEntityCell<'w>) -> Box<[DynamicItem<'w>]> {
    let mut item_buffer = Vec::with_capacity(fetches.len());
    for fetch in unsafe { fetches.iter(entity) } {
        item_buffer.push(fetch);
    }
    item_buffer.into()
}
pub struct RoDynamicQueryIter<'w, 's> {
    entities: &'w [ArchetypeEntity],
    world: UnsafeWorldCell<'w>,
    fetch: &'s Fetches,
    filter: &'s Filters,
    ids: iter::Map<Ones<'s>, fn(usize) -> ArchetypeId>,
    buffer: Option<Box<[DynamicItem<'w>]>>,
    ticks: Ticks,
}
impl<'w, 's> RoDynamicQueryIter<'w, 's> {
    /// Get next entity.
    ///
    /// Advances `entities` and `query_archetype` as much as necessary to get to
    /// the next entity.
    ///
    /// Returns `None` if we exhausted all entities present in `query_archetypes`.
    fn next_entity(&mut self) -> Option<UnsafeEntityCell<'w>> {
        loop {
            let Some((first, remaining)) = self.entities.split_first() else {
                let Some(next_archetype) = self.ids.next() else {
                    return None;
                };
                let archetype = self.world.archetypes().get(next_archetype);
                let archetype = unsafe { archetype.prod_unchecked_unwrap() };
                self.entities = archetype.entities();
                continue;
            };
            self.entities = remaining;

            let entity = self.world.get_entity(first.entity());
            let entity = unsafe { entity.prod_unchecked_unwrap() };

            let mut conjunctions = self.filter.tick_conjunctions(self.ticks);
            if conjunctions.any(|c| c.within_tick(entity)) {
                return Some(entity);
            }
        }
    }
    pub fn new(world: UnsafeWorldCell<'w>, state: &'s DynamicState, ticks: Ticks) -> Self {
        let mut this = Self {
            ids: state.archetype_ids.iter(),
            filter: &state.filters,
            fetch: &state.fetches,
            entities: &[][..],
            buffer: None,
            world,
            ticks,
        };
        if let Some(next_entity) = this.next_entity() {
            this.buffer = Some(fetch_buffer_ro(this.fetch, next_entity));
        }
        this
    }
}
impl<'w, 's> Iterator for RoDynamicQueryIter<'w, 's> {
    // TODO(perf): Get rid of individual allocation per iteration.
    type Item = Box<[DynamicItem<'w>]>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.buffer.take()?;
        let Some(entity) = self.next_entity() else {
            return Some(ret);
        };
        self.buffer = Some(fetch_buffer_ro(self.fetch, entity));
        Some(ret)
    }
}

pub struct DynamicQueryIter<'w, 's>(RoDynamicQueryIter<'w, 's>);
impl<'w, 's> DynamicQueryIter<'w, 's> {
    pub fn new(world: UnsafeWorldCell<'w>, state: &'s DynamicState, ticks: Ticks) -> Self {
        let mut this = RoDynamicQueryIter {
            ids: state.archetype_ids.iter(),
            filter: &state.filters,
            fetch: &state.fetches,
            entities: &[][..],
            buffer: None,
            world,
            ticks,
        };
        if let Some(next_entity) = this.next_entity() {
            this.buffer = Some(fetch_buffer(this.fetch, next_entity));
        }
        Self(this)
    }
}
impl<'w, 's> Iterator for DynamicQueryIter<'w, 's> {
    type Item = Box<[DynamicItem<'w>]>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.buffer.take()?;
        let Some(entity) = self.0.next_entity() else {
            return Some(ret);
        };
        self.0.buffer = Some(fetch_buffer(self.0.fetch, entity));
        Some(ret)
    }
}
