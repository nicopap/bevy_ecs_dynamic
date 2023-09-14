# Query Interpreter demo

The query interpreter example demonstrates the absolute flexibility of
dynamic queries. Indeed you can just type in the input field a query and it
will… query the world.

## Explanation

The screen has two input fields, a yellow one, and a blue one.

The outline of the input field is red if they contain invalid text, they are
green if they contain valid text.

In the background, a simple scene with moving colored cubes and spheres.

### Yellow input field

It accepts a single "update expression". It will be applied to all mutable
items accessed by the query whenever Enter is pressed while the blue input
field is focused.

## Grammar

To demonstrate the dynamic nature of `DynamicQuery`, we use a text-based
grammar.

### Expression grammar

Nothing more trivial. It only accepts changing the value of a numerical field,
or setting a value of a field with a reflected type.

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
