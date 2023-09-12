use core::fmt;

use bevy_ecs::query::{ReadOnlyWorldQuery, WorldQuery};
use bevy_ecs::{component::ComponentId, prelude::Query, world::World};
use bevy_reflect::ReflectFromPtr;

use crate::DynamicQuery;
use traits::{DFetches, DOr};

pub use methods::DynamicQueryBuilder;
pub use traits::DQuery;

mod methods;
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
#[derive(Clone, Default, Debug)]
pub struct OrFilters(pub Vec<AndFilters>);

#[derive(Clone)]
pub struct FetchData {
    pub id: ComponentId,
    pub from_ptr: ReflectFromPtr,
}
impl fmt::Debug for FetchData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FetchData").field(&self.id).finish()
    }
}

#[derive(Clone, Debug)]
pub enum Fetch {
    Read(FetchData),
    Mut(FetchData),
    OptionRead(FetchData),
    OptionMut(FetchData),
    Entity,
}
impl Fetch {
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
    pub(crate) fn data(&self) -> &FetchData {
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match self {
            Read(data) | Mut(data) | OptionRead(data) | OptionMut(data) => data,
            Fetch::Entity => {
                unreachable!("Entity Fetch cannot be duplicated (probably several encountered)")
            }
        }
    }
}
impl PartialEq for Fetch {
    fn eq(&self, other: &Self) -> bool {
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match (self, other) {
            (Read(left), Read(right))
            | (Mut(left), Mut(right))
            | (OptionRead(left), OptionRead(right))
            | (OptionMut(left), OptionMut(right)) => left.id == right.id,
            (Fetch::Entity, Fetch::Entity) => true,
            _ => false,
        }
    }
}
impl Eq for Fetch {}
impl PartialOrd for Fetch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::{Equal, Greater, Less};
        use Fetch::{Mut, OptionMut, OptionRead, Read};
        match (self, other) {
            (Read(left), Read(right))
            | (Mut(left), Mut(right))
            | (OptionRead(left), OptionRead(right))
            | (OptionMut(left), OptionMut(right)) => left.id.partial_cmp(&right.id),
            (Fetch::Entity, Fetch::Entity) => Some(Equal),
            (Mut(_), Read(_))
            | (OptionRead(_), Read(_) | Mut(_))
            | (OptionMut(_), Read(_) | Mut(_) | OptionRead(_))
            | (Fetch::Entity, _) => Some(Greater),
            _ => Some(Less),
        }
    }
}
impl Ord for Fetch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<Q, F> DQuery for Query<'_, '_, Q, F>
where
    Q: WorldQuery + DFetches,
    F: ReadOnlyWorldQuery + DOr,
{
    fn dynamic(world: &mut World) -> DynamicQuery {
        let fetches = Q::fetches(world);
        let filters = F::or(world);
        DynamicQuery::new(fetches, filters).unwrap()
    }
}
