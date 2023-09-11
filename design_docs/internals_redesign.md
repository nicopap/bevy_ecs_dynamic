## Why the previous version sucked and is difficult to review

It's just a copy/paste of the pre-existing code for the typed queries, with
a `enum` layer on top.

## What can be improved

Now this is odd. We can do much better.

The code for typed queries (`WorldQuery` impls) have for particularity that
they must deal with the fact they don't have any knowledge of the global `Query`
structure, only its individual elements.

No such issue with a dynamic queries. Since we don't need to do the
`WorldQuery` dance, we just need to identify the archetypes we are matching
and go from that point.

## More improvements

- Split archetypes listing on conjunctions, so that we don't have to check the
  archetype on conjunction to check for the correct `Or` conjunction.

### Metadata rediction

We keep around a lot of metadata as tag:

```rust
enum Baz {
  X(u32),
  Y(u32),
  Z(u32),
}
struct FooBar {
  bazes: Vec<Baz>,
}
```

This requires checking on tag and (potentially) differently on each individual
element. This introduces a lot of branching where none is needed.

None is needed?

Yeah, we could discriminate first on variant and then proceed to compute
on them:

```rust
struct FooBar {
  xs: Vec<u32>,
  ys: Vec<u32>,
  zs: Vec<u32>,
}
```

Now we don't have branching. But we still have loop checking, and we are getting
a very fat overhead (24 bits) _per_ entry, total of 72 bits. If we only have
at most 2 or 3 `u32`s it gets expensive.

We could use `ArrayVec` to inline everything. Though it still has a `u32` for
length overhead, and we lose flexibility, as we get a maximum bound on component
count.

Now, with a constructor, we could get the perfect design.

However, this is at the cost of dynamism. So there is a fundamental limitation
of dynamic queries.

```rust
trait List<Ty> {
  const LEN: usize;
  fn get(&self, index: usize) -> Option<&Ty>;
  fn get_mut(&mut self, index: usize) -> Option<&mut Ty>;
}
impl<Hd> List<Hd> for () {
  const LEN: usize = 0;

  #[inline(always)]
  fn get(&self, _: usize) -> Option<&Hd> { None }
  #[inline(always)]
  fn get_mut(&self, _: usize) -> Option<&Hd> { None }
}
impl<Hd, Tl: List<Hd>> List<Hd> for Flist<Hd, Tl> {
  const LEN: usize = Tl::LEN + 1;

  #[inline(always)]
  fn get(&self, index: usize) -> Option<&Hd> {
    if index == 0 { Some(&self.0) } else { self.1.get(index - 1) }
  }
  #[inline(always)]
  fn get_mut(&mut self, index: usize) -> Option<&mut Hd> {
    if index == 0 { Some(&mut self.0) } else { self.1.get_mut(index - 1) }
  }
}
trait ToArray<T>: List<Ty> {
  fn to_array(self) -> [T; Self::LEN];
}
macro_rules! to_array_impl {
  (@item[$($head:tt)*] Flist(h, $tail:expr)) => { to_array_impl!(@item[$($head)*, h] $tail) }
  (@item[$($full_list:tt)*] ()) => { [ $($full_list)* ] }
  ( (): () ) => {
    impl<T> ToArray<T> for () {
      fn to_array(self) -> [T; Self::LEN] { [] }
    }
  };
  ( Flist(h, $tail:expr): Flist<H, $ty_tail:ty> ) => {
    impl<H> ToArray<H> for Flist<H, $ty_tail:ty> {
      fn to_array(self) -> [H; Self::LEN] { to_array_impl!(@item[h] $tail) }
    }
    to_array_impl!{$tail : $ty_tail}
  }
}
to_array_impl! {
  Flist(h, Flist(h, Flist(h, Flist(h, Flist(h, Flist(h, Flist(h, Flist(h, Flist(h, ()))))))))):
  Flist<H, Flist<H, Flist<H, Flist<H, Flist<H, Flist<H, Flist<H, Flist<H, Flist<H, ()>>>>>>>>>
}

struct Flist<Hd, Tl>(Hd, Tl);
impl<Hd, Tl: List<Hd>> for Flist<Hd, Tl> {
  fn add(self, head: Hd) -> Flist<Hd, Flist<Hd, Tl>> {
    Self(head, self)
  }
}
enum FetchSource {
  Entity,
  SetRead,
  SetMut,
  SetOptRead,
  SetOptMut,
  TableRead,
  TableMut,
  TableOptRead,
  TableOptMut,
}
struct FetchMap {
  source: FetchSource,
  index: u32,
}
type Id = ComponentId;
struct Fetches<R: List<Id>, M: List<Id>, OR: List<Id>, OM: List<Id>> {
  read: R,
  mut: M,
  opt_read: OR,
  opt_mut: OM,
}
struct NotDynamicQueryState<SetFtch, TableFtch, FtchMp: List<FetchMap>, Entty = ()> {
  entity: Entty,
  sets: SetFtch,
  tables: TableFtch,
  fetch_mapping: FtchMp,
}
impl<SR, SM, SOR, SOM, TR, TM, TOR, TOM, M: List<FetchMap>>
  NotDynamicQueryState<Fetches<SR, SM, SOR, SOM>, Fetches<TR, TM, TOR, TOM>, M, ()> {

  fn add_table_read_fetch(self, id: Id) -> NotDynamicQueryState<
    Fetches< Flist<Id, SR>, SM, SOR, SOM >,
    Fetches< TR, TM, TOR, TOM >,
    Flist<Id, M>,
    (),
  > {
    NotDynamicQueryState {
      entity: (),
      sets: self.sets,
      tables: self.tables.add_read(id),
      fetch_mapping: self.fetch_mapping.add_read(id),
    }
  }
  // Duplicate this 8 time, once per column
}
```

This design fails to de-dupcliate redundant components.

But the idea is sound.

If we don't have a `Changed` filter, or `Option` filter, the logic for it
doesn't exist, since we generated a struct with a `()` for `opt_read`
or `changed` fields.