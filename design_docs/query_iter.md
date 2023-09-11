# Reading note on the bevy implementation

```rust
// NOTE: If you are changing query iteration code, remember to update the following places, where relevant:
// QueryIter, QueryIterationCursor, QueryManyIter, QueryCombinationIter, QueryState::for_each_unchecked_manual, QueryState::par_for_each_unchecked_manual
/// # Safety
/// `tables` and `archetypes` must belong to the same world that the [`QueryIterationCursor`]
/// was initialized for.
/// `query_state` must be the same [`QueryState`] that was passed to `init` or `init_empty`.
#[inline(always)]
unsafe fn next(
    &mut self,
    tables: &'w Tables,
    archetypes: &'w Archetypes,
    query_state: &'s QueryState<Q, F>,
) -> Option<Q::Item<'w>> {
    if Self::IS_DENSE {
loop {
    // we are on the beginning of the query, or finished processing a table, so skip to the next
    if self.current_row == self.current_len {
        let table_id = self.table_id_iter.next()?;
        let table = tables.get(*table_id).debug_checked_unwrap();
        Q::set_table(&mut self.fetch, &query_state.fetch_state, table);
        F::set_table(&mut self.filter, &query_state.filter_state, table);
        self.table_entities = table.entities();
        self.current_len = table.entity_count();
        self.current_row = 0;
        continue;
    }
    let entity = *self.table_entities.get_unchecked(self.current_row);
    let row = TableRow::new(self.current_row);
    // End elided, because identical to other
}
    } else {
loop {
    if self.current_row == self.current_len {
        let archetype_id = self.archetype_id_iter.next()?;
        let archetype = archetypes.get(*archetype_id).debug_checked_unwrap();
        let table = tables.get(archetype.table_id()).debug_checked_unwrap();
        Q::set_archetype(&mut self.fetch, &query_state.fetch_state, archetype, table);
        F::set_archetype(&mut self.filter, &query_state.filter_state, archetype, table);
        self.archetype_entities = archetype.entities();
        self.current_len = archetype.len();
        self.current_row = 0;
        continue;
    }
    let archetype_entity = self.archetype_entities.get_unchecked(self.current_row);
    let entity = archetype_entity.entity();
    let row = archetype_entity.table_row();
    if !F::filter_fetch(&mut self.filter, entity, row) {
        self.current_row += 1;
        continue;
    }
    let item = Q::fetch(&mut self.fetch, entity, row);
    self.current_row += 1;
    return Some(item);
}
    }
}
```

Two things that I need to get out:

- We see some egregious code duplication
- We are using some fields when `Self::IS_DENSE` and not when not. Meaning the
  `Cursor` type is bloated, probably causes register contention.

Supposedly, the fast path is the first brach (`IS_DENSE`).
This means, the thing being avoided is:

- `WorldQuery::set_archetype`
- `archetypes.get(id)`

Why? Looking at `set_archetype` impls:

- On `&`, `&mut`: It calls `Self::set_table` if `Self::IS_DENSE`, otherwise
  does nothing. (`Self::IS_DENSE = Component::STORAGE_TYPE == StorageType::Table`)
- On `Or<(…)>`, `Option<T>`:
    - sets `self.matches` to `T::matches_component_set(|id| archetype.contains(id))`
    - Note that `WorldQuery::set_table` does as much with
      `T::matches_component_set(|id| table.has_column(id))`
- `matches_component_set` just calls the thing passed in argument with the component id.

## Data structures

```rust
pub struct SparseSet<I, V: 'static> {
    dense: Vec<V>,
    indices: Vec<I>,
    sparse: SparseArray<I, usize>,
}
pub struct Table {
    columns: Sparse { ComponentId -> Column },
    entities: Vec<Entity>,
}
pub struct Column {
    data: BlobVec,
    added_ticks: Vec<UnsafeCell<Tick>>,
    changed_ticks: Vec<UnsafeCell<Tick>>,
}
// Contains a Column, but rows are not ordered.
pub struct ComponentSparseSet {
    Entity -> ColumnRow
}
pub struct SparseSets {
    sets: Sparse { ComponentId -> ComponentSparseSet}
}
```

# Our implementation

## Initial implementation

We can just store the list of entities in archetype with the `UnsafeWorldCell`,
and query each individual entity one after another.

## Very fast (like very) implementation

The idea is to have a `FetchCursor` type, one per `Fetches` field.

- For `SparseSet` components, store the `ComponentSparseSet`.
- For `Table` components, `Ptr` + `usize` stride
- A `&[Entity]` is stored separately, remove head each step to keep track of how
  far in the `Table` we are for table components & know the current entity.

On end of `Table`, we need to update the `Table` `FetchCursor`s for the next
archetype table.

