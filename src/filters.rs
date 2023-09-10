use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::{ComponentId, Tick};
use bevy_ecs::storage::{Table, TableRow};
use fixedbitset::FixedBitSet;

use crate::debug_unchecked::DebugUnchecked;
use crate::fetches::Fetches;
use crate::jagged_array::JaggedArray;

// TODO(BUG): Must ensure correct sort order when creating.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filters(JaggedArray<Filter>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum FilterKind {
    With = 0,
    Changed = 1,
    Added = 2,
    Without = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Filter {
    component: u32,
}

impl FilterKind {
    const fn from_u32(u32: u32) -> Self {
        match u32 {
            0 => FilterKind::With,
            1 => FilterKind::Changed,
            2 => FilterKind::Added,
            3 => FilterKind::Without,
            _ => unreachable!(),
        }
    }
}
impl Filter {
    const MASK: u32 = 0x7f_ff_ff_ff;
    const KIND_OFFSET: u32 = 30;

    pub const fn id(&self) -> ComponentId {
        let masked = self.component & Self::MASK;
        ComponentId::new(masked as usize)
    }
    pub const fn kind(&self) -> FilterKind {
        let unmasked = self.component >> Self::KIND_OFFSET;
        FilterKind::from_u32(unmasked)
    }
    pub fn new(kind: FilterKind, id: ComponentId) -> Self {
        let kind_mask = (kind as u32) << Self::KIND_OFFSET;
        let id_mask = id.index() as u32;
        let component = kind_mask | id_mask;
        Filter { component }
    }
}

/// [`Filters`] are a list of "conjunction".
pub struct Conjunction<'a> {
    filters: &'a [Filter],
    fetches: &'a Fetches,
}

struct InclusiveFilter<'a>(&'a [Filter]);
struct ExclusiveFilter<'a>(&'a [Filter]);
struct ChangedFilter<'a>(&'a [Filter]);
struct AddedFilter<'a>(&'a [Filter]);

impl Filters {
    pub fn conjunctions<'a>(
        &'a self,
        fetches: &'a Fetches,
    ) -> impl Iterator<Item = Conjunction<'a>> + 'a {
        let conjunction = |filters| Conjunction { filters, fetches };
        self.0.rows_iter().map(conjunction)
    }
    pub fn tick_conjunctions<'a>(
        &'a self,
        last_run: Tick,
        this_run: Tick,
    ) -> impl Iterator<Item = TickConjunction<'a>> + 'a {
        let conjunction = move |filters| TickConjunction { filters, last_run, this_run };
        self.0.rows_iter().map(conjunction)
    }
}
fn tick_filters<'a>(filters: &'a [Filter]) -> (ChangedFilter<'a>, AddedFilter<'a>) {
    // A Filter value that always fit at the very end of the inclusive filters range.
    let mut last_with = Filter::new(FilterKind::Changed, ComponentId::new(0));
    let mut last_changed = Filter::new(FilterKind::Added, ComponentId::new(0));
    let mut last_added = Filter::new(FilterKind::Without, ComponentId::new(0));
    last_with.component -= 1;
    last_changed.component -= 1;
    last_added.component -= 1;

    let first_changed = filters.binary_search(&last_with).err();
    let first_added = filters.binary_search(&last_changed).err();
    let first_without = filters.binary_search(&last_added).err();

    let first_changed = unsafe { first_changed.release_unchecked_unwrap() };
    let first_added = unsafe { first_added.release_unchecked_unwrap() };
    let first_without = unsafe { first_without.release_unchecked_unwrap() };

    let changed_filter = ChangedFilter(&filters[first_changed..first_added]);
    let added_filter = AddedFilter(&filters[first_added..first_without]);
    (changed_filter, added_filter)
}
fn filters<'a>(filters: &'a [Filter]) -> (InclusiveFilter<'a>, ExclusiveFilter<'a>) {
    // A Filter value that always fit at the very end of the inclusive filters range.
    let mut last_inclusive = Filter::new(FilterKind::Without, ComponentId::new(0));
    last_inclusive.component -= 1;

    // SAFETY: If we find a componet with an ID equal to 2**30, something fishy is going on
    let first_exclusive = filters.binary_search(&last_inclusive).err();
    let first_exclusive = unsafe { first_exclusive.release_unchecked_unwrap() };
    let (inclusive, exclusive) = filters.split_at(first_exclusive);
    (InclusiveFilter(inclusive), ExclusiveFilter(exclusive))
}
impl<'a> Conjunction<'a> {
    // O(n²) where n is sizeof archetype
    pub fn includes_archetype(&self, archetype: &Archetype) -> bool {
        // NOTE(perf): We don't skip this on `fetch_archetype == false` because
        // we hope the optimizer can merge `all_included` `for` with this one.
        let (inclusive, exclusive) = filters(self.filters);
        let include_filter = inclusive.all_included(archetype.components());
        let exclude_filter = exclusive.any_excluded(archetype.components());
        let fetch_archetype = self.fetches.all_included(archetype.components());

        fetch_archetype && include_filter && !exclude_filter
    }
}
/// [`Filters`] are a list of "conjunction"
pub struct TickConjunction<'a> {
    filters: &'a [Filter],
    last_run: Tick,
    this_run: Tick,
}
impl<'a> TickConjunction<'a> {
    // NOTE: unlike `fetches::FetchesIter::next`, we can't assume we are on the
    // right table, because we may call this with a table from a different conjunction.
    // TODO(perf): This needs to be cached.
    // O(n²) where n is sizeof archetype
    pub fn within_tick(&self, archetype: &Archetype, table: &Table, row: TableRow) -> bool {
        let (inclusive, exclusive) = filters(self.filters);
        let include_filter = inclusive.all_included(archetype.components());
        let exclude_filter = exclusive.any_excluded(archetype.components());

        if !include_filter || exclude_filter {
            return false;
        }
        let (changed, added) = tick_filters(self.filters);
        let changed = changed.within_tick(table, row, self.last_run, self.this_run);
        let added = added.within_tick(table, row, self.last_run, self.this_run);

        changed && added
    }
}
// TODO(perf): Likely can avoid O(n²). If only `ComponedId`s were
// ordered in `Archetype::components()`…
impl<'a> InclusiveFilter<'a> {
    // TODO(BUG): Doesn't work with repeated components. We need to ensure this
    // in `Fetches` constructor.
    #[inline]
    pub fn all_included(self, ids: impl Iterator<Item = ComponentId>) -> bool {
        let mut found = FixedBitSet::with_capacity(self.0.len());
        for id in ids {
            if let Some(idx) = self.0.iter().position(|x| x.id() == id) {
                found.set(idx, true);
            }
        }
        found.count_ones(..) == self.0.len()
    }
}
impl<'a> ExclusiveFilter<'a> {
    #[inline]
    pub fn any_excluded(&self, ids: impl Iterator<Item = ComponentId>) -> bool {
        let mut found = false;
        for id in ids {
            found |= self.0.binary_search_by_key(&id, Filter::id).is_ok();
        }
        found
    }
}
impl<'a> ChangedFilter<'a> {
    fn within_tick(&self, table: &Table, row: TableRow, last_run: Tick, this_run: Tick) -> bool {
        self.0
            .iter()
            .map(|f| unsafe { table.get_column(f.id()).release_unchecked_unwrap() })
            .map(|c| unsafe { c.get_changed_ticks_unchecked(row) })
            .all(|t| unsafe { t.get().read().is_newer_than(last_run, this_run) })
    }
}
impl<'a> AddedFilter<'a> {
    fn within_tick(&self, table: &Table, row: TableRow, last_run: Tick, this_run: Tick) -> bool {
        self.0
            .iter()
            .map(|f| unsafe { table.get_column(f.id()).release_unchecked_unwrap() })
            .map(|c| unsafe { c.get_added_ticks_unchecked(row) })
            .all(|t| unsafe { t.get().read().is_newer_than(last_run, this_run) })
    }
}
