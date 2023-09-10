# Example grammar

To demonstrate the dynamic nature of `DynamicQuery`, we use a text-based
grammar. The grammar in question is specified as follow:

```ungrammar
Query       = 'Query' '<' Fetches   (',' OrFilters)?   ','? '>'
OrFilters   = 'Or' '<(' Conjunction (',' Conjunction)* ','? ')>'
Fetches     = Fetch  | '(' Fetch  (',' Fetch)*  ','? ')'
Conjunction = Filter | '(' Filter (',' Filter)* ','? ')'
Fetch
  = '&'      'ident'
  | '&mut'   'ident'
  | 'Option' '<' '&'    'ident' '>'
  | 'Option' '<' '&mut' 'ident' '>'
  | 'Has'    '<' 'ident' '>'
Filter
  = 'With'    '<' 'ident' '>'
  | 'Added'   '<' 'ident' '>'
  | 'Changed' '<' 'ident' '>'
  | 'Without' '<' 'ident' '>'
```

Examples:

```rust
Query<
  (&mut Health, &Damage, Option<&Armor>),
  Or<(
    (With<Player>, Without<Invincible>, Changed<Damage>),
    (With<Enemy>, Without<FirstBossYouAreMeantToDieFrom>, Changed<Damage>),
  )>
>
```

