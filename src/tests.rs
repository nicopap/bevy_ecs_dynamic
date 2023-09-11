use bevy::prelude::*;
use cuicui_dsl::{dsl, DslBundle};

use crate::DynamicQuery;

//
// table
//

// not registered, table

#[derive(Component)]
struct TableNorgTag;
#[derive(Component)]
struct TableNorgSimple {
    x: usize,
}
#[derive(Component)]
struct TableNorgFancy {
    zoo: Box<usize>,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

// registered, table

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct TableRegTag;
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct TableRegSimple {
    x: usize,
}
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct TableRegFancy {
    zoo: usize,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

//
// sparse set
//

// not registered, sparse set

#[derive(Component)]
#[component(storage = "SparseSet")]
struct SetNorgTag;
#[derive(Component)]
#[component(storage = "SparseSet")]
struct SetNorgSimple {
    x: usize,
}
#[derive(Component)]
#[component(storage = "SparseSet")]
struct SetNorgFancy {
    zoo: Box<usize>,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

// registered, sparse set

#[derive(Component, Reflect, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegTag;
#[derive(Component, Reflect, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegSimple {
    x: usize,
}
#[derive(Component, Reflect, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegFancy {
    zoo: usize,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

// - validate `Table` and `SparseSet` components
// - both in filter and fetch positions
// - how we handle multiple of same component
// - how we handle unregistered components
// - how we handle archetype updates
// - how we handle optional components (`Table` and `SparseSet`)
// - how we handle Or
// - Iteration
// - Changed / Added
// - Without
// - With
// - Get: present/not present

fn make_query() -> DynamicQuery {
    let dynamic_query = DynamicQuery::from_query::<
        Query<
            (&SetRegTag, Option<&mut TableRegFancy>),
            Or<(
                Changed<Transform>,
                (Without<Transform>, Added<SetRegSimple>),
            )>,
        >,
    >(cs, registry);
}
