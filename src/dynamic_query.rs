use bevy_ecs::prelude::{Entity, World};
use bevy_reflect::Reflect;

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
    pub fn new(fetches: Vec<Fetch>, filters: OrFilters) -> Option<Self> {
        let fetches = Fetches::new(fetches)?;
        let filters = Filters::new(filters)?;
        Some(DynamicQuery { fetches, filters })
    }
    pub fn state(&self, world: &mut World) -> DynamicState {
        DynamicState::in_world(self, world)
    }
    /// Build a `DynamicQuery` with the same shape as the `Q` `Query`.
    ///
    /// # Examples
    ///
    /// ```
    /// let dynamic_query = DynamicQuery::from_query::<
    ///     Query<(&mut Transform, &Sprite), With<Player>>,
    /// >(&mut world);
    /// ```
    ///
    /// # Panics
    /// - `world` doesn't have an `AppTypeRegistry`
    /// - any component in [`DQuery`] are not reflect-registered.
    pub fn from_query<Q: DQuery>(world: &mut World) -> Self {
        Q::dynamic(world)
    }
}
