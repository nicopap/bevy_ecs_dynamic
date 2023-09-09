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