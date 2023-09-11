use bevy_ecs::component::Components;
use bevy_ecs::prelude::{Entity, World};
use bevy_reflect::{Reflect, TypeRegistry};

use crate::{fetches::Fetches, filters::Filters, DQuery, DynamicState, Fetch, OrFilters};

pub enum DynamicItem<'a> {
    Entity(Entity),
    Read(&'a dyn Reflect),
    Mut(&'a mut dyn Reflect),
    OptionRead(Option<&'a dyn Reflect>),
    OptionMut(Option<&'a mut dyn Reflect>),
}

#[derive(Clone, Debug)]
pub struct DynamicQuery {
    pub(crate) fetches: Fetches,
    pub(crate) filters: Filters,
}

impl DynamicQuery {
    pub fn new(fetches: Vec<Fetch>, filters: OrFilters, registry: &TypeRegistry) -> Option<Self> {
        let fetches = Fetches::new(fetches, registry)?;
        let filters = Filters::new(filters)?;
        Some(DynamicQuery { fetches, filters })
    }
    pub fn state(&self, world: &mut World) -> DynamicState {
        DynamicState::in_world(self, world)
    }
    pub fn from_query<Q: DQuery>(cs: &Components, registry: &TypeRegistry) -> Self {
        Q::dynamic(cs, registry)
    }
}
