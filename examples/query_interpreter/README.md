# Query Interpreter demo

```sh
cargo run -p query_interpreter
```

The query interpreter example demonstrates the absolute flexibility of
dynamic queries. Indeed you can just type in the input field a query and it
will… query the world.

## Explanation

The screen has two input fields, a yellow one, and a blue one.

- Yellow input field: accepts assignments separated by commas
- Blue input field: accepts a single query

The outline of the input field is red if they contain invalid text, they are
green if they contain valid text.

In the background, a simple scene with moving colored cubes and spheres.

### Yellow input field

It accepts "update expressions" separated by commas. It will be applied to all mutable
items accessed by the query whenever Enter is pressed while the blue input
field is focused.

## Grammar

To demonstrate the dynamic nature of `DynamicQuery`, we use a text-based
grammar.

### Assignment grammar

Assignment is:

1. A path as per bevy's `GetPath` system
2. An assignment operator (one of `=`, `-=`, `+=`, `*=`, `/=`)
3. A value. Arithmetic assignment only accept numerical values, while the bare
   assignment operator accepts any value. It uses Ron deserialization based on
   bevy reflect deserialization.

Formally, the grammar is as follow:

```ungrammar
Expression = Path Update
Path = PathElement (PathElement)*
PathElement
   = '[' 'ident' ']'
   | '.' 'ident'
Update
   = '=' TokenTree
   | '-=' 'ident'
   | '+=' 'ident'
   | '*=' 'ident'
   | '/=' 'ident'
```

Examples:

```rust
.scale.x *= 10
.overflow.x = Visible
.sections[0].style.font_size = 32
.sections[1].value = "Hello world"
```

### Query grammar

It is the list of `Fetch` items, followed, optionally by a list of filters.

```ungrammar
Query = Fetches (',' OrFilters)?
OrFilters = Conjunction ('|' Conjunction)*
Fetches = (Fetch)*
Conjunction = (Filter)*
Fetch
   = '?' 'ident'    // Option<&ident>
   | '?mut' 'ident' // Option<&mut ident>
   | 'mut' 'ident'  // &mut ident
   | 'Entity'       // Entity
   | 'ident'        // &ident
Filter
   = '+' 'ident' // Added
   | '>' 'ident' // Changed
   | '!' 'ident' // Without
   | 'ident'     // With
```

Examples:

```
// 1
Fetch
// 2
Fetch, Filter1 | Filter2 | Filter3
// 3
Fetch1 Fetch2
// 4
Fetch1 mut Fetch2 ?Fetch3, WithFilter1 !WithoutFilter1 | WithFilter2 !WithoutFilter2
// 5
mut Health Damage ?Armor,
  Player !Invincible >Damage | Enemy !FirstBossYouAreMeantToDieFrom >Damage
```

We also accept a more rusty grammar. All the following work and are equivalent
to the previous code block:

```rust
// 1
Query<&Fetch>
// 2
Query<&Fetch, Or<(Filter1, Filter2, Filter3)>>
// 3
Query<(&Fetch1, &Fetch2)>
// 4
Query<
   (&Fetch1, &mut Fetch2, Option<&Fetch3>),
   Or<(
      (With<WithFilter1>, Without<WithoutFilter1>),
      (With<WithFilter2>, Without<WithoutFilter2>),
   )>,
>
// 5
Query<
   (&mut Health, &Damage, Option<&Armor>),
   Or<(
      (With<Player>, Without<Invincible>, Changed<Damage>),
      (With<Enemy>, Without<FirstBossYouAreMeantToDieFrom>, Changed<Damage>),
   )>,
>
```

The terse and less readable option exists because it's difficult to input text
using the rudimentary input widget we implement.
