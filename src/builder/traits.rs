pub use impls::*;

#[allow(unused_parens)]
#[rustfmt::skip]
mod impls {

use crate::builder::{Fetch, AndFilter, OrFilters, AndFilters};
use crate::DynamicQuery;
use bevy_ecs::{prelude::*, all_tuples};
use bevy_ecs::component::{Component as Comp, Components as Cs, ComponentId, ComponentInfo};
use bevy_reflect::TypeRegistry;

fn with_info<'s, C: Component, O>(
    cs: &'s Cs,
    f: impl FnOnce(&'s ComponentInfo) -> O + 's,
) -> O {
    let id = cs.component_id::<C>().unwrap();
    f(cs.get_info(id).unwrap())
}
fn with_id<C: Component, O>(cs: &Cs, f: impl FnOnce(ComponentId) -> O) -> O {
    f(cs.component_id::<C>().unwrap())
}

trait DFilter     { fn filter (cs: &Cs) -> AndFilter; }
trait DAnd        { fn and    (cs: &Cs) -> AndFilters; }
trait DFetch      { fn fetch  (cs: &Cs) -> Fetch; }
pub trait DOr     { fn or     (cs: &Cs) -> OrFilters; }
pub trait DFetches{ fn fetches(cs: &Cs) -> Vec<Fetch>; }
pub trait DQuery  { fn dynamic(cs: &Cs, registry: &TypeRegistry) -> DynamicQuery; }

impl<C: Comp> DFetch for &'_ C             { fn fetch(cs: &Cs) -> Fetch      { with_info::<C, _>(cs, Fetch::Read) } }
impl<C: Comp> DFetch for &'_ mut C         { fn fetch(cs: &Cs) -> Fetch      { with_info::<C, _>(cs, Fetch::Mut) } }
impl<C: Comp> DFetch for Option<&'_ C>     { fn fetch(cs: &Cs) -> Fetch      { with_info::<C, _>(cs, Fetch::OptionRead) } }
impl<C: Comp> DFetch for Option<&'_ mut C> { fn fetch(cs: &Cs) -> Fetch      { with_info::<C, _>(cs, Fetch::OptionMut) } }
impl<C: Comp> DFilter for With<C>          { fn filter(cs: &Cs) -> AndFilter { with_id::<C, _>(cs, AndFilter::With) } }
impl<C: Comp> DFilter for Without<C>       { fn filter(cs: &Cs) -> AndFilter { with_id::<C, _>(cs, AndFilter::Without) } }
impl<C: Comp> DFilter for Changed<C>       { fn filter(cs: &Cs) -> AndFilter { with_id::<C, _>(cs, AndFilter::Changed) } }
impl<C: Comp> DFilter for Added<C>         { fn filter(cs: &Cs) -> AndFilter { with_id::<C, _>(cs, AndFilter::Added) } }
impl<T: DAnd> DOr for T { fn or(cs: &Cs) -> OrFilters { OrFilters(vec![<Self as DAnd>::and(cs)]) }}

macro_rules! impl_dfetches {
    ($($T:ident),*) => {
impl<$($T : DFetch),*> DFetches for ($($T),*) {
    fn fetches(cs: &Cs) -> Vec<Fetch> { vec![ $( <$T as DFetch>::fetch(cs), )* ] }
} }; }
macro_rules! impl_andfilters {
    ($($T:ident),*) => {
impl<$($T : DFilter),*> DAnd for ($($T),*) {
    fn and(cs: &Cs) -> AndFilters { AndFilters(vec![ $( <$T as DFilter>::filter(cs), )* ]) }
} }; }
macro_rules! impl_orfilters {
    ($($T:ident),*) => {
impl<$($T : DAnd),*> DOr for Or<($($T),*)> {
    fn or(cs: &Cs) -> OrFilters { OrFilters(vec![ $( <$T as DAnd>::and(cs), )* ]) }
} }; }
all_tuples!(impl_dfetches, 1, 13, T);
all_tuples!(impl_orfilters, 2, 13, T);
all_tuples!(impl_andfilters, 1, 13, T);
}
