use std::fmt::Debug;
use std::str::FromStr;

use bevy::prelude::*;
use bevy_ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy_ecs::{component::StorageType, system::SystemState};
use cuicui_dsl::{dsl, DslBundle};
use pretty_assertions::assert_str_eq;
use test_log::test;

use crate::builder::{DFetches, DOr};
use crate::pretty_print::{DynShow, DynShowT, ShowReflect};
use crate::{DQuery, DynamicQuery, DynamicQueryBuilder};
use dy_cmp::{Dyeq, Dyown};

mod dy_cmp;

#[derive(Clone, Copy, Debug, Default)]
enum Complexity {
    #[default]
    Tag,
    Simple,
    Fancy,
}
#[derive(Clone, Debug, Default)]
struct SpawnInstruction {
    storage: StorageType,
    registered: bool,
    complexity: Complexity,
}
impl SpawnInstruction {
    fn insert(&mut self, cmds: &mut cuicui_dsl::EntityCommands) {
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
    }
}
impl FromStr for SpawnInstruction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split('-').collect::<Vec<_>>();
        Ok(SpawnInstruction {
            storage: match splits[0] {
                "set" => StorageType::SparseSet,
                "table" => StorageType::Table,
                "" => default(),
                _ => return Err(()),
            },
            registered: match splits[1] {
                "reg" => true,
                "unreg" => false,
                "" => default(),
                _ => return Err(()),
            },
            complexity: match splits[2] {
                "tag" => Complexity::Tag,
                "simple" => Complexity::Simple,
                "fancy" => Complexity::Fancy,
                "" => default(),
                _ => return Err(()),
            },
        })
    }
}
#[derive(Clone, Debug, Default)]
struct Dsl {
    to_spawn: Vec<SpawnInstruction>,
}
impl Dsl {
    fn kind(&mut self, spec: &str) {
        self.to_spawn.push(spec.parse().unwrap());
    }
}
impl DslBundle for Dsl {
    fn insert(&mut self, cmds: &mut cuicui_dsl::EntityCommands) -> Entity {
        for mut to_spawn in self.to_spawn.drain(..) {
            to_spawn.insert(cmds);
        }
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

#[derive(Component, Clone, PartialEq, Reflect, Debug, Default)]
#[reflect(Component)]
struct TableRegTag;
#[derive(Component, Clone, PartialEq, Reflect, Debug, Default)]
#[reflect(Component)]
struct TableRegSimple {
    x: usize,
}
#[derive(Component, Clone, Debug, PartialEq, Reflect, Default)]
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

#[derive(Component, Clone, PartialEq, Reflect, Debug, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegTag;
#[derive(Component, Clone, PartialEq, Reflect, Debug, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
struct SetRegSimple {
    x: usize,
}
#[derive(Component, Clone, PartialEq, Reflect, Debug, Default)]
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

fn make_query2(world: &mut World) -> DynamicQuery {
    DynamicQueryBuilder::new(world)
        .component::<SetRegTag>()
        .optional_mut::<TableRegFancy>()
        .or(|b| b.changed::<Transform>())
        .or(|b| b.without::<Transform>().added::<SetRegSimple>())
        .build()
        .unwrap()
}
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
                spawn(kind("-reg-")) {
                    spawn(kind("set--")) {
                        spawn(kind("set-reg-tag"), kind("table-reg-fancy"));
                        spawn(kind("set-reg-fancy"), kind("table-reg-tag"));
                        spawn(kind("set-reg-simple"), kind("table-reg-fancy"));

                        spawn(kind("table-reg-tag"), kind("set-reg-tag"));
                        spawn(kind("table-reg-fancy"), kind("set-reg-fancy"));
                        spawn(kind("table-reg-simple"), kind("set-reg-simple"));
                    }
                    spawn(kind("table--")) {
                        spawn(kind("table-reg-tag"));
                        spawn(kind("table-reg-fancy"));
                        spawn(kind("table-reg-simple"));
                    }
                }
                spawn(kind("-unreg-")) {
                    spawn(kind("set--")) {
                        spawn(kind("set-unreg-tag"));
                        spawn(kind("set-unreg-fancy"));
                        spawn(kind("set-unreg-simple"));
                    }
                    spawn(kind("table--")) {
                        spawn(kind("table-unreg-tag"));
                        spawn(kind("table-unreg-fancy"));
                        spawn(kind("table-unreg-simple"));
                    }
                }
            }
        }
        state.apply(&mut world);
    }
    world
}

#[track_caller]
fn test_query_get<DQ: DQuery>(
    world: &mut World,
    entity: Entity,
    expected: impl ShowReflect + Dyeq,
) {
    let query = DynamicQuery::from_query::<DQ>(world);
    let mut state = query.state(world);
    let value = state.get_mut(world, entity).unwrap();

    let equivalent = expected.dyeq(value);
    if !equivalent {
        let expected = format!("{:?}", DynShowT(&expected));
        let actual = format!("{:?}", DynShow::new(value));
        assert_str_eq!(expected, actual);
    }
}
#[track_caller]
fn test_single_entity<Q, F>(mut world: World)
where
    Q: DFetches + WorldQuery,
    for<'w> Q::Item<'w>: Dyown,
    for<'w> <Q::Item<'w> as Dyown>::Owned: Dyeq + ShowReflect + 'static,
    F: DOr + ReadOnlyWorldQuery,
{
    let world = world.as_unsafe_world_cell();

    let results = {
        let first_world = unsafe { world.world_mut() };
        let mut fancy_entity = first_world.query_filtered::<(Entity, Q), F>();
        fancy_entity
            .iter_mut(first_world)
            .map(|(e, o)| (e, o.own()))
            .collect::<Vec<_>>()
    };
    assert!(!results.is_empty());
    {
        let second_world = unsafe { world.world_mut() };
        for (entity, owned) in results.into_iter() {
            test_query_get::<Query<Q, F>>(second_world, entity, owned)
        }
    }
}
#[test]
fn simple_table_query() {
    test_single_entity::<&TableRegFancy, ()>(test_world());
}
#[test]
fn simple_sparse_query() {
    test_single_entity::<&SetRegFancy, ()>(test_world());
}
#[test]
fn two_fetch_sparse_query() {
    test_single_entity::<(&SetRegFancy, &mut TableRegTag), ()>(test_world());
}
#[test]
fn with_query() {
    test_single_entity::<&SetRegFancy, With<TableRegFancy>>(test_world());
}
#[test]
fn or_query() {
    test_single_entity::<
        (Option<&SetRegFancy>, Option<&mut TableRegFancy>),
        Or<(With<SetRegFancy>, With<TableRegFancy>)>,
    >(test_world());
}
