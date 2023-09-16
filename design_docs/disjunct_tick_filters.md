# How to handle disjunciton on `Changed` filters?

**Problem**: Queries that have several disjunct filters on archetype, with an
additional filter on tick. Example:

```rust
Query<&mut Damage, Or<((With<Player>, Changed<Armor>), (With<Enemy>, Changed<Damage>))>>,
```

Why is this a problem? Suppose we have an archetype-only query. We could just
merge all the archetypes in the archetype list. Cool.

Suppose we have a query without disjunctions, only a single filter:

```rust
Query<&mut Damage, (With<Player>, Changed<Armor>)>,
```

We can simply read all archetypes in the archetype list, and then check the
`Armor` change tick. All is good.

But with the disjunct tick filter. We can't. Why is that?

We need to limit our `Changed<Armor>` check to archetypes fullfilling `With<Player>`,
suppose `Enemy` has an `Armor` component, we certainly don't want to to filter-out
enemies which `Armor` didn't change.

## Solution

Current approach is to match the archetype in each conjunction, which is very
costly.

A better approach is to:

1. Detect if we hit a disjunct tick filter
2. If so, store the individual archetypes matched by disjunctions in a `JaggedBitset`
   (blessed be my socks for implementing this in a completely different context)
   This replaces the `MatchedArchetypes` unified `Bitset` actually…
3. When iterating, go through each disjunct filter one after another
4. When `get`-ing, we need to check the archetype in each disjunction.
