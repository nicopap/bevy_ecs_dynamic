use std::fmt::Debug;

use bevy::prelude::*;
use bevy_ecs::{component::StorageType, system::SystemState};
use cuicui_dsl::{dsl, DslBundle};
use test_log::test;

use crate::{DQuery, DynamicQuery};

use self::dy_cmp::Dyeq;

mod dy_cmp;
mod pretty_print;

#[derive(Clone, Copy, Debug, Default)]
enum Complexity {
    #[default]
    Tag,
    Simple,
    Fancy,
}
#[derive(Clone, Debug, Default)]
struct Dsl {
    storage: StorageType,
    registered: bool,
    complexity: Complexity,
}
#[rustfmt::skip]
impl Dsl {
    fn table(&mut self)  { self.storage = StorageType::Table; }
    fn set(&mut self)    { self.storage = StorageType::SparseSet; }

    fn reg(&mut self)    { self.registered = true; }
    fn unreg(&mut self)  { self.registered = false; }

    fn tag(&mut self)    { self.complexity = Complexity::Tag; }
    fn simple(&mut self) { self.complexity = Complexity::Simple; }
    fn fancy(&mut self)  { self.complexity = Complexity::Fancy; }
}
impl DslBundle for Dsl {
    fn insert(&mut self, cmds: &mut cuicui_dsl::EntityCommands) -> Entity {
        use Complexity::{Fancy, Simple, Tag};
        use StorageType::{SparseSet, Table};
        #[rustfmt::skip]
        match (self.storage, self.registered, self.complexity) {
            (Table,     true,  Tag)    => cmds.insert(TableRegTag),
            (Table,     true,  Simple) => cmds.insert(TableRegSimple::default()),
            (Table,     true,  Fancy)  => cmds.insert(TableRegFancy::default()),
            (Table,     false, Tag)    => cmds.insert(TableNorgTag),
            (Table,     false, Simple) => cmds.insert(TableNorgSimple::default()),
            (Table,     false, Fancy)  => cmds.insert(TableNorgFancy::default()),
            (SparseSet, true,  Tag)    => cmds.insert(SetRegTag),
            (SparseSet, true,  Simple) => cmds.insert(SetRegSimple::default()),
            (SparseSet, true,  Fancy)  => cmds.insert(SetRegFancy::default()),
            (SparseSet, false, Tag)    => cmds.insert(SetNorgTag),
            (SparseSet, false, Simple) => cmds.insert(SetNorgSimple::default()),
            (SparseSet, false, Fancy)  => cmds.insert(SetNorgFancy::default()),
        };
        cmds.id()
    }
}

//
// table
//

// not registered, table

#[derive(Component, Debug, Default)]
struct TableNorgTag;
#[derive(Component, Debug, Default)]
struct TableNorgSimple {
    x: usize,
}
#[derive(Component, Debug, Default)]
struct TableNorgFancy {
    zoo: Box<usize>,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

// registered, table

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct TableRegTag;
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct TableRegSimple {
    x: usize,
}
#[derive(Component, Debug, PartialEq, Reflect, Default)]
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

#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
struct SetNorgTag;
#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
struct SetNorgSimple {
    x: usize,
}
#[derive(Component, Debug, Default)]
#[component(storage = "SparseSet")]
struct SetNorgFancy {
    zoo: Box<usize>,
    bar: Vec<Transform>,
    entity: Option<Entity>,
}

// registered, sparse set

#[derive(Component, Reflect, Debug, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegTag;
#[derive(Component, Reflect, Debug, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegSimple {
    x: usize,
}
#[derive(Component, Reflect, Debug, Default)]
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

fn make_query(world: &mut World) -> DynamicQuery {
    DynamicQuery::from_query::<
        Query<
            (&SetRegTag, Option<&mut TableRegFancy>),
            Or<(
                Changed<Transform>,
                (Without<Transform>, Added<SetRegSimple>),
            )>,
        >,
    >(world)
}

fn test_world() -> World {
    let mut world = World::new();
    let reg = AppTypeRegistry::default();
    {
        let mut reg = reg.write();
        reg.register::<Vec<Transform>>();
        reg.register::<Option<Entity>>();
        reg.register::<SetRegTag>();
        reg.register::<SetRegSimple>();
        reg.register::<SetRegFancy>();
        reg.register::<TableRegTag>();
        reg.register::<TableRegSimple>();
        reg.register::<TableRegFancy>();
    }
    world.insert_resource(reg);
    {
        let mut state = SystemState::<Commands>::new(&mut world);
        let mut cmds = state.get_mut(&mut world);
        dsl! {
            cmds,
            spawn() {
                spawn(reg) {
                    spawn(set) {
                        spawn(reg, set, tag);
                        spawn(reg, set, fancy);
                        spawn(reg, set, simple);
                    }
                    spawn(table) {
                        spawn(reg, table, tag);
                        spawn(reg, table, fancy);
                        spawn(reg, table, simple);
                    }
                }
                spawn(unreg) {
                    spawn(set) {
                        spawn(set, tag);
                        spawn(set, fancy);
                        spawn(set, simple);
                    }
                    spawn(table) {
                        spawn(table, tag);
                        spawn(table, fancy);
                        spawn(table, simple);
                    }
                }
            }
        }
        state.apply(&mut world);
    }
    world
}

#[track_caller]
fn test_query_get<DQ: DQuery>(world: &mut World, entity: Entity, expected: impl Dyeq + Debug) {
    let query = DynamicQuery::from_query::<DQ>(world);
    let mut state = query.state(world);
    let value = state.get(&world, entity).unwrap();

    assert!(expected.dyeq(value))
}
#[test]
fn simple_table_query() {
    let mut world = test_world();
    let mut fancy_entity = world.query_filtered::<Entity, With<TableRegFancy>>();
    let fancy_entity = fancy_entity.get_single(&world).unwrap();

    let mut f = TableRegFancy::default();
    f.zoo += 1;
    test_query_get::<Query<&TableRegFancy>>(&mut world, fancy_entity, &f)
}
