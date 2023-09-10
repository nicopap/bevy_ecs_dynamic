# Architecture

The API offered by this crate is as follow:

- `DynamicQuery`: A runtime query _description_. This can be built from an
  arbitrary set of `ComponentId`.
- `DynamicQueryState`: Can be created from a `DynamicQuery`. With this, you can
  run the query on a `World` and get back components or iterator over components.

`DynamicQueryState` is a bit more restrictive than the bevy `QueryState`. Notably:

- It doesn't allow duplicate items in `fetch` position
- It only allows `Entity` as query parameter in first position
- In `filter` position, the `Or`/`And` conditions must be in [disjunctive
  normal form][dnf] (basically it means there is a single OR, and it must be
  the very top level of the filter)
- It only allows a single item to exist concurrently, even read-only items
## Crate structure

### `ctor_dsl`

A DSL to build a `DynamicQuery` with ease.

### `fetches`

Handles the `fetch` side of the query.

### `filters`

Handles the `filter` side of the query. This includes `Added` and `Changed`
query parameters.

We use a `JaggedArray` to store the distinct [conjunctions][dnf] of the filter.


### `iter`

Handles the iterators returned by the `DynamicQueryState::iter[_mut]` methods.

### `state`

`DynamicQueryState` definition.

[dnf]: https://en.wikipedia.org/wiki/Disjunctive_normal_form
