pub use impls::*;

// `all_tuples!` for `()` and `(foo)` generate those warnings
#[allow(unused_parens, unused_variables)]
#[rustfmt::skip]
mod impls {

use crate::builder::{Fetch, AndFilter, OrFilters, AndFilters, FetchData};
use crate::DynamicQuery;
use bevy_ecs::{prelude::*, all_tuples};
use bevy_ecs::component::{Component as Comp, ComponentId};
use bevy_reflect::ReflectFromPtr;

// TODO(err): do not panic on missing registration, instead return error.
fn with_info<C: Component, O>(
    world: &mut World,
    f: impl FnOnce(FetchData) -> O,
) -> O {
    let id = world.init_component::<C>();
    let type_id = std::any::TypeId::of::<C>();
    let registry = world.resource::<AppTypeRegistry>().read();
    let from_ptr = registry.get_type_data::<ReflectFromPtr>(type_id).unwrap().clone();
    f(FetchData { id, from_ptr })
}
fn with_id<C: Component, O>(world: &mut World, f: impl FnOnce(ComponentId) -> O) -> O {
    f(world.init_component::<C>())
}

trait DFilter     { fn filter (w: &mut World) -> AndFilter; }
trait DAnd        { fn and    (w: &mut World) -> AndFilters; }
trait DFetch      { fn fetch  (w: &mut World) -> Fetch; }
pub trait DOr     { fn or     (w: &mut World) -> OrFilters; }
pub trait DFetches{ fn fetches(w: &mut World) -> Vec<Fetch>; }
pub trait DQuery  { fn dynamic(world: &mut World) -> DynamicQuery; }

impl<C: Comp> DFetch for &'_ C             { fn fetch(w: &mut World) -> Fetch      { with_info::<C, _>(w, Fetch::Read) } }
impl<C: Comp> DFetch for &'_ mut C         { fn fetch(w: &mut World) -> Fetch      { with_info::<C, _>(w, Fetch::Mut) } }
impl<C: Comp> DFetch for Option<&'_ C>     { fn fetch(w: &mut World) -> Fetch      { with_info::<C, _>(w, Fetch::OptionRead) } }
impl<C: Comp> DFetch for Option<&'_ mut C> { fn fetch(w: &mut World) -> Fetch      { with_info::<C, _>(w, Fetch::OptionMut) } }
impl<C: Comp> DFilter for With<C>          { fn filter(w: &mut World) -> AndFilter { with_id::<C, _>(w, AndFilter::With) } }
impl<C: Comp> DFilter for Without<C>       { fn filter(w: &mut World) -> AndFilter { with_id::<C, _>(w, AndFilter::Without) } }
impl<C: Comp> DFilter for Changed<C>       { fn filter(w: &mut World) -> AndFilter { with_id::<C, _>(w, AndFilter::Changed) } }
impl<C: Comp> DFilter for Added<C>         { fn filter(w: &mut World) -> AndFilter { with_id::<C, _>(w, AndFilter::Added) } }
impl<T: DAnd> DOr for T { fn or(w: &mut World) -> OrFilters { OrFilters(vec![<Self as DAnd>::and(w)]) }}

macro_rules! impl_dfetches { ($($T:ident),*) => {

impl<$($T : DFetch),*> DFetches for ($($T),*) {
    fn fetches(w: &mut World) -> Vec<Fetch> { vec![ $( <$T as DFetch>::fetch(w), )* ] }
} }; }
macro_rules! impl_andfilters { ($($T:ident),*) => {

impl<$($T : DFilter),*> DAnd for ($($T),*) {
    fn and(w: &mut World) -> AndFilters { AndFilters(vec![ $( <$T as DFilter>::filter(w), )* ]) }
} }; }
macro_rules! impl_orfilters { ($($T:ident),*) => {

impl<$($T : DAnd),*> DOr for Or<($($T),*)> {
    fn or(w: &mut World) -> OrFilters { OrFilters(vec![ $( <$T as DAnd>::and(w), )* ]) }
} }; }
all_tuples!(impl_dfetches, 0, 13, T);
all_tuples!(impl_orfilters, 2, 13, T);
all_tuples!(impl_andfilters, 0, 13, T);
}
