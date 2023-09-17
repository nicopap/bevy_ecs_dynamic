use bevy_ecs::{
    archetype::{ArchetypeId, Archetypes},
    world::unsafe_world_cell::UnsafeEntityCell,
};
use datazoo::bitset::{Bitset, Ones};

use crate::{
    fetches::Fetches,
    filters::{Conjunction, Conjunctions, Filters},
    Ticks,
};

fn u32_to_archetype_id(u32: u32) -> ArchetypeId {
    unsafe { std::mem::transmute(u32) }
}
fn archetype_id_to_u32(id: ArchetypeId) -> u32 {
    // SAFETY: ArchetypeId is repr(transparent) u32
    unsafe { std::mem::transmute(id) }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MatchedArchetypes {
    ids: Bitset<Box<[u32]>>,
    // A cool trick here is that if `tick_match` has zero matches, the `Box` stores in `len`
    // reference metadata that it has size 0.
    // Which avoid an indirection when checking if we have tick_matches.
    tick_matches: Bitset<Box<[u32]>>,
    last_gen: usize,
}
impl MatchedArchetypes {
    pub(crate) fn new(fetches: &Fetches, filters: &Filters, archetypes: &Archetypes) -> Self {
        let mut this = MatchedArchetypes::default();
        this.add_archetypes(fetches, filters, archetypes);
        this
    }
    pub(crate) fn add_archetypes(
        &mut self,
        fetches: &Fetches,
        filters: &Filters,
        archetypes: &Archetypes,
    ) {
        // TODO(perf): check this actually skips things.
        for archetype in archetypes.iter().skip(self.last_gen) {
            let (matches, ticks) = if filters.is_empty() {
                (fetches.all_included(archetype.components()), false)
            } else {
                let match_and_tick = |(mut matches, mut tick), conj: Conjunction| {
                    let included = conj.includes(fetches, archetype);
                    matches |= included;
                    tick |= included & conj.has_tick_filter();
                    (matches, tick)
                };
                filters.conjunctions().fold((false, false), match_and_tick)
            };
            let id = archetype_id_to_u32(archetype.id()) as usize;
            if matches {
                self.ids.enable_bit_extending(id);
            }
            if ticks {
                self.tick_matches.enable_bit_extending(id);
            }
        }
        self.last_gen = archetypes.len();
    }
}
impl MatchedArchetypes {
    #[inline]
    pub(crate) fn getter<'a>(&'a self, filters: &'a Filters) -> ArchematchGet<'a> {
        ArchematchGet {
            ids: Bitset(&self.ids.0),
            tick_matches: Bitset(&self.tick_matches.0),
            filters,
        }
    }
    #[inline]
    pub(crate) fn iter<'a>(&'a self, filters: &'a Filters) -> ArchematchIter<'a> {
        ArchematchIter {
            ids: self.ids.ones(),
            tick_matches: Bitset(&self.tick_matches.0),
            filters: filters.conjunctions(),
        }
    }
}

pub(crate) struct ArchematchGet<'a> {
    ids: Bitset<&'a [u32]>,
    tick_matches: Bitset<&'a [u32]>,
    filters: &'a Filters,
}
impl<'a> ArchematchGet<'a> {
    #[inline]
    pub(crate) fn contains(&self, ticks: Ticks, entity: UnsafeEntityCell) -> bool {
        let archetype = entity.archetype();
        let id = archetype_id_to_u32(archetype.id()) as usize;

        // Not matched in any case, caputz
        if !self.ids.bit(id) {
            return false;
        }
        // No additional tick-based filter
        if !self.tick_matches.bit(id) {
            return true;
        }
        let conjunctions = self.filters.conjunctions();
        let mut conjunctions = conjunctions
            .filter(|c| c.has_tick_filter())
            .filter(|c| c.in_filter(archetype));

        conjunctions.any(|c| c.within_tick(ticks, entity))
    }
}
#[derive(Default)]
enum CheckTickInner {
    #[default]
    None,
    Filter,
}
#[derive(Default)]
pub(crate) struct CheckTick(CheckTickInner);

impl CheckTick {
    pub(crate) fn within_tick(
        &self,
        iter: &ArchematchIter,
        ticks: Ticks,
        entity: UnsafeEntityCell,
    ) -> bool {
        match self.0 {
            CheckTickInner::None => true,
            CheckTickInner::Filter => {
                let conjs = iter.filters.clone();
                let archetype = entity.archetype();

                let mut conjunctions = conjs
                    .filter(|c| c.has_tick_filter())
                    .filter(|c| c.in_filter(archetype));

                conjunctions.any(|c| c.within_tick(ticks, entity))
            }
        }
    }
}
pub(crate) struct ArchematchIter<'a> {
    ids: Ones<'a>,
    tick_matches: Bitset<&'a [u32]>,
    filters: Conjunctions<'a>,
}
impl<'a> Iterator for ArchematchIter<'a> {
    type Item = (ArchetypeId, CheckTick);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.ids.next()?;

        let check = if self.tick_matches.bit(next as usize) {
            CheckTick(CheckTickInner::Filter)
        } else {
            CheckTick(CheckTickInner::None)
        };
        Some((u32_to_archetype_id(next), check))
    }
}
