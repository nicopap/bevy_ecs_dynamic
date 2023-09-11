use bevy_ecs::component::{ComponentId, ComponentInfo, Components};
use bevy_ecs::prelude::Query;
use bevy_ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy_reflect::TypeRegistry;

use crate::DynamicQuery;
use traits::{DFetches, DOr};

pub use traits::DQuery;

mod traits;

#[derive(Clone, Copy, Debug)]
pub enum AndFilter {
    With(ComponentId),
    Without(ComponentId),
    Changed(ComponentId),
    Added(ComponentId),
}
#[derive(Clone, Debug)]
pub struct AndFilters(pub Vec<AndFilter>);
#[derive(Clone, Debug)]
pub struct OrFilters(pub Vec<AndFilters>);

#[derive(Clone, Copy, Debug)]
pub enum Fetch<'a> {
    Read(&'a ComponentInfo),
    Mut(&'a ComponentInfo),
    OptionRead(&'a ComponentInfo),
    OptionMut(&'a ComponentInfo),
    Entity,
}
impl Fetch<'_> {
    // SAFETY: !!!!!IMPORTANT!!!! Make sure this is in the same order as
    // the enum variants in `Fetch`.
    pub(crate) const READ_IDX: usize = 0;
    pub(crate) const MUT_IDX: usize = 1;
    pub(crate) const OPTION_READ_IDX: usize = 2;
    pub(crate) const OPTION_MUT_IDX: usize = 3;
    pub(crate) const ENTITY_IDX: usize = 4;

    pub(crate) const fn discriminant_index(&self) -> usize {
        match self {
            Fetch::Read(_) => Fetch::READ_IDX,
            Fetch::Mut(_) => Fetch::MUT_IDX,
            Fetch::OptionRead(_) => Fetch::OPTION_READ_IDX,
            Fetch::OptionMut(_) => Fetch::OPTION_MUT_IDX,
            Fetch::Entity => Fetch::ENTITY_IDX,
        }
    }
    pub(crate) fn info(&self) -> &ComponentInfo {
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match self {
            Read(comp) | Mut(comp) | OptionRead(comp) | OptionMut(comp) => comp,
            Fetch::Entity => {
                unreachable!("Entity Fetch cannot be duplicated (probably several encountered)")
            }
        }
    }
}
impl PartialEq for Fetch<'_> {
    fn eq(&self, other: &Self) -> bool {
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match (self, other) {
            (Read(left), Read(right))
            | (Mut(left), Mut(right))
            | (OptionRead(left), OptionRead(right))
            | (OptionMut(left), OptionMut(right)) => left.id() == right.id(),
            (Fetch::Entity, Fetch::Entity) => true,
            _ => false,
        }
    }
}
impl Eq for Fetch<'_> {}
impl PartialOrd for Fetch<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::{Equal, Greater, Less};
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match (self, other) {
            (Read(left), Read(right))
            | (Mut(left), Mut(right))
            | (OptionRead(left), OptionRead(right))
            | (OptionMut(left), OptionMut(right)) => left.id().partial_cmp(&right.id()),
            (Fetch::Entity, Fetch::Entity) => Some(Equal),
            (Mut(_), Read(_))
            | (OptionRead(_), Read(_) | Mut(_))
            | (OptionMut(_), Read(_) | Mut(_) | OptionRead(_))
            | (Fetch::Entity, _) => Some(Greater),
            _ => Some(Less),
        }
    }
}
impl Ord for Fetch<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<Q, F> DQuery for Query<'_, '_, Q, F>
where
    Q: WorldQuery + DFetches,
    F: ReadOnlyWorldQuery + DOr,
{
    fn dynamic(components: &Components, registry: &TypeRegistry) -> DynamicQuery {
        DynamicQuery::new(Q::fetches(components), F::or(components), registry).unwrap()
    }
}
