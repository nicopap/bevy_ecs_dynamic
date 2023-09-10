use bevy_ecs::prelude::{Entity, World};
use bevy_ecs::storage::TableRow;
use bevy_reflect::Reflect;

use crate::fetches::{Fetch, Fetches};
use crate::filters::Filters;
use crate::DynamicState;
use crate::OrFilters;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Row {
    pub(crate) entity: Entity,
    pub(crate) row: TableRow,
}

#[derive(Default)]
pub enum DynamicItem<'a> {
    #[default]
    Uninitialized,
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
    pub fn new(fetches: &[Fetch], filters: OrFilters) -> Self {
        todo!()
    }
    pub fn state(&self, world: &mut World) -> DynamicState {
        todo!()
    }
}
