use bevy_ecs::{
    component::ComponentId,
    prelude::{Component, World},
    reflect::AppTypeRegistry,
};
use bevy_reflect::ReflectFromPtr;

use crate::DynamicQuery;

use super::{AndFilter, AndFilters, Fetch, FetchData, OrFilters};

pub struct OrBuilder<'w> {
    world: &'w mut World,
    filters: AndFilters,
}
pub struct DynamicQueryBuilder<'w> {
    world: &'w mut World,
    fetches: Vec<Fetch>,
    filters: OrFilters,
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
            filters: OrFilters(Vec::new()),
        }
    }

    pub fn or(
        &mut self,
        f: impl for<'a, 'z> FnOnce(&'a mut OrBuilder<'z>) -> &'a mut OrBuilder<'z>,
    ) -> &mut Self {
        let mut conjunction = OrBuilder { world: self.world, filters: AndFilters(Vec::new()) };
        f(&mut conjunction);
        self.filters.0.push(conjunction.filters);
        self
    }
    pub fn component<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(self.world);
        self.ref_by_id(data)
    }

    pub fn component_mut<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(self.world);
        self.mut_by_id(data)
    }

    pub fn ref_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::Read(info));
        self
    }

    pub fn mut_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::Mut(info));
        self
    }

    pub fn optional<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(self.world);
        self.optional_ref_by_id(data)
    }

    pub fn optional_mut<T: Component>(&mut self) -> &mut Self {
        let data = with_info::<T>(self.world);
        self.optional_mut_by_id(data)
    }

    pub fn optional_ref_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::OptionRead(info));
        self
    }

    pub fn optional_mut_by_id(&mut self, info: FetchData) -> &mut Self {
        self.fetches.push(Fetch::OptionMut(info));
        self
    }

    pub fn build(&mut self) -> Option<DynamicQuery> {
        use std::mem::take;
        DynamicQuery::new(take(&mut self.fetches), take(&mut self.filters))
    }
}

impl<'w> OrBuilder<'w> {
    pub fn with<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.with_by_id(data)
    }

    pub fn without<T: Component>(&mut self) -> &mut Self {
        let data = self.world.init_component::<T>();
        self.without_by_id(data)
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

    pub fn added_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::Added(id));
        self
    }

    pub fn changed_by_id(&mut self, id: ComponentId) -> &mut Self {
        self.filters.0.push(AndFilter::Changed(id));
        self
    }
}
