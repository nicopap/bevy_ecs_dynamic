use bevy_ecs::component::{ComponentId, Components};
use bevy_reflect::{ReflectFromPtr, TypeRegistry};

use super::{AndFilter, AndFilters, Fetch, FetchData, OrFilters};
use crate::DynamicQuery;

pub struct NamedDynamicBuilder<'w> {
    comps: &'w Components,
    reg: &'w TypeRegistry,
    fetches: Vec<Fetch>,
    filters: OrFilters,
}

pub struct NamedOrBuilder<'w> {
    comps: &'w Components,
    reg: &'w TypeRegistry,
    filters: AndFilters,
}

fn with_id(registry: &TypeRegistry, comps: &Components, name: impl AsRef<str>) -> ComponentId {
    // TODO(err): should return result instead.
    let registration = registry.get_with_short_name(name.as_ref()).unwrap();
    let type_id = registration.type_id();
    comps.get_id(type_id).unwrap()
}
fn with_info(registry: &TypeRegistry, comps: &Components, name: impl AsRef<str>) -> FetchData {
    // TODO(err): should return result instead.
    let registration = registry.get_with_short_name(name.as_ref()).unwrap();
    let from_ptr = registration.data::<ReflectFromPtr>().unwrap().clone();
    let type_id = registration.type_id();
    let id = comps.get_id(type_id).unwrap();
    FetchData { id, from_ptr }
}

impl<'w> NamedDynamicBuilder<'w> {
    pub fn new(reg: &'w TypeRegistry, comps: &'w Components) -> Self {
        Self {
            reg,
            comps,
            fetches: Vec::new(),
            filters: OrFilters(Vec::new()),
        }
    }

    pub fn or(
        &mut self,
        f: impl for<'a, 'z> FnOnce(&'a mut NamedOrBuilder<'z>) -> &'a mut NamedOrBuilder<'z>,
    ) -> &mut Self {
        let mut conjunction = NamedOrBuilder {
            reg: self.reg,
            comps: self.comps,
            filters: AndFilters(Vec::new()),
        };
        f(&mut conjunction);
        self.filters.0.push(conjunction.filters);
        self
    }
    pub fn component(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_info(self.reg, self.comps, name);
        self.ref_by_id(data)
    }

    pub fn component_mut(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_info(self.reg, self.comps, name);
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

    pub fn optional(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_info(self.reg, self.comps, name);
        self.optional_ref_by_id(data)
    }

    pub fn optional_mut(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_info(self.reg, self.comps, name);
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

    pub fn entity(&mut self) -> &mut Self {
        self.fetches.push(Fetch::Entity);
        self
    }

    pub fn build(&mut self) -> Option<DynamicQuery> {
        use std::mem::take;
        DynamicQuery::new(take(&mut self.fetches), take(&mut self.filters))
    }
}

impl<'w> NamedOrBuilder<'w> {
    pub fn with(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_id(self.reg, self.comps, name);
        self.with_by_id(data)
    }

    pub fn without(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_id(self.reg, self.comps, name);
        self.without_by_id(data)
    }

    pub fn added(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_id(self.reg, self.comps, name);
        self.added_by_id(data)
    }

    pub fn changed(&mut self, name: impl AsRef<str>) -> &mut Self {
        let data = with_id(self.reg, self.comps, name);
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
