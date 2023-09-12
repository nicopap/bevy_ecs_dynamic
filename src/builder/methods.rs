use bevy_ecs::{
    component::ComponentId,
    prelude::{Component, World},
    reflect::AppTypeRegistry,
};
use bevy_reflect::ReflectFromPtr;

use crate::DynamicQuery;

use super::{AndFilter, AndFilters, Fetch, FetchData, OrFilters};

pub struct DynamicQueryBuilder<'w> {
    world: &'w mut World,
    fetches: Vec<Fetch>,
    filters: AndFilters,
}
// TODO(err): do not panic on missing registration, instead return error.
fn with_info<C: Component>(world: &mut World) -> FetchData {
    let id = world.init_component::<C>();
    let type_id = std::any::TypeId::of::<C>();
    let registry = world.resource::<AppTypeRegistry>().read();
    let from_ptr = registry
        .get_type_data::<ReflectFromPtr>(type_id)
        .unwrap()
        .clone();
    FetchData { id, from_ptr }
}

impl<'w> DynamicQueryBuilder<'w> {
    pub fn new(world: &'w mut World) -> Self {
        Self {
            world,
            fetches: Vec::new(),
            filters: AndFilters(Vec::new()),
        }
    }

    pub fn with<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.with_by_id(data)
    }

    pub fn without<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.without_by_id(data)
    }

    pub fn component<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(&mut self.world);
        self.ref_by_id(data)
    }

    pub fn component_mut<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(&mut self.world);
        self.mut_by_id(data)
    }

    pub fn added<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.added_by_id(data)
    }

    pub fn changed<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.changed_by_id(data)
    }

    pub fn with_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::With(id));
        self
    }

    pub fn without_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::Without(id));
        self
    }

    pub fn ref_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::Read(info));
        self
    }

    pub fn mut_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::Mut(info));
        self
    }

    pub fn added_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::Added(id));
        self
    }

    pub fn changed_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::Changed(id));
        self
    }

    pub fn build(self) -> Option<DynamicQuery> {
        DynamicQuery::new(self.fetches, OrFilters(vec![self.filters]))
    }
}
