# bevy_mod_dynamic_query

Fork of <https://github.com/jakobhellermann/bevy_ecs_dynamic>, a prototype
for dynamic queries in bevy.

`bevy_ecs_dynamic` was severly out of date and was missing a few query parameters:

- `Or<(…)>`
- `Option<Component>`
- Some other kind of queries that are a combinations of the previous

In logic, we can always express a logical expression as a [disjunction of
conjunctions][dnf]. So we can use a `Vec<Vec<Filter>>` to express `Or`s.

[dnf]: https://en.wikipedia.org/wiki/Disjunctive_normal_form

We can always provide an API that accepts a arbitrary logic expression and
flatten it if necessary.

### External API draft

Suppose we have a script that wants to query anything.

```javascript
function damage_system(q) {
  for (item in q) {
    let [health, damage, opt_armor] = item;
    var real_damage = damage;
    if (opt_armor != null) {
      if (opt_armor > damage) {
        real_damage -= opt_armor;
      } else {
        real_damage = 1;
      }
    }
    health.set(health.get() - real_damage);
  }
}
// equivalent to:
// Query<
//     (&mut Health, &Damage, Option<&Armor>),
//     Or<(
//         (With<Player>, Without<Invicible>),
//         (With<Enemy>, Without<FirstBossYouAreMeantToDieFrom>),
//     )>
// >,
var my_query = {
  fetch: [
    mut("Health"),
    read("Damage"),
    read_nullable("Armor"),
  ],
  filter: or(
    and("Player", not("Invicible")),
    and("Enemy", not("FirstBossYouAreMeantToDieFrom")),
  ),
};
runtime.register_system(my_query, damage_system);
```

