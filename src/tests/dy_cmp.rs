use std::fmt;

use bevy_ecs::{all_tuples, prelude::Entity, prelude::Mut as BevyMut};
use bevy_reflect::Reflect;

use crate::{pretty_print::ShowReflect, DynamicItem};

pub trait Dyown {
    type Owned;

    fn own(&self) -> Self::Owned;
}
impl Dyown for Entity {
    type Owned = Entity;
    fn own(&self) -> Self::Owned {
        *self
    }
}
impl<T: PartialEq + Reflect + Clone> Dyown for &'_ T {
    type Owned = Ref<T>;

    fn own(&self) -> Self::Owned {
        Ref(T::clone(self))
    }
}
impl<T: PartialEq + Reflect + Clone> Dyown for BevyMut<'_, T> {
    type Owned = Mut<T>;

    fn own(&self) -> Self::Owned {
        Mut(T::clone(self))
    }
}
impl<T: PartialEq + Reflect + Clone> Dyown for Option<&'_ T> {
    type Owned = OptRef<T>;

    fn own(&self) -> Self::Owned {
        OptRef(self.cloned())
    }
}
impl<T: PartialEq + Reflect + Clone> Dyown for Option<BevyMut<'_, T>> {
    type Owned = OptMut<T>;

    fn own(&self) -> Self::Owned {
        OptMut(self.as_ref().map(|x| T::clone(x)))
    }
}

/// Compare something to a `&[DynamicItem]`.
pub trait Dyeq {
    fn dyeq(&self, items: &[DynamicItem]) -> bool;
}
pub trait DyeqItem {
    fn dyeq_item(&self, item: &DynamicItem) -> bool;
}
pub struct Ref<T>(T);
pub struct Mut<T>(T);
pub struct OptRef<T>(Option<T>);
pub struct OptMut<T>(Option<T>);
impl<T: PartialEq + Reflect> DyeqItem for Ref<T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Read(r) if r.downcast_ref() == Some(&self.0))
    }
}
impl<T: PartialEq + Reflect> DyeqItem for Mut<T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Mut(r) if r.downcast_ref() == Some(&self.0))
    }
}
impl DyeqItem for Entity {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Entity(e) if e == self)
    }
}
impl<T: PartialEq + Reflect> DyeqItem for OptRef<T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        use DynamicItem::OptionRead;
        let opt: Option<Option<&T>> = self.0.as_ref().map(Some);
        matches!(item, OptionRead(r) if r.map(<dyn Reflect>::downcast_ref) == opt)
    }
}
impl<T: PartialEq + Reflect> DyeqItem for OptMut<T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        use DynamicItem::OptionMut;
        let opt: Option<Option<&T>> = self.0.as_ref().map(Some);
        matches!(item, OptionMut(r) if r.as_ref().map(|r| r.downcast_ref()) == opt)
    }
}
impl<T: Reflect> ShowReflect for Ref<T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("&")?;
        self.0.debug(f)
    }
}
impl<T: Reflect> ShowReflect for Mut<T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("&mut ")?;
        self.0.debug(f)
    }
}
impl<T: Reflect> ShowReflect for OptRef<T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = &self.0 {
            f.write_str("Some(&")?;
            value.debug(f)?;
            f.write_str(")")
        } else {
            f.write_str("None")
        }
    }
}
impl<T: Reflect> ShowReflect for OptMut<T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = &self.0 {
            f.write_str("Some(&mut ")?;
            value.debug(f)?;
            f.write_str(")")
        } else {
            f.write_str("None")
        }
    }
}

macro_rules! impl_dyown {
    ($(($Tn:ident, $t_n:ident)),*) => {
        impl<$($Tn : Dyown),*> Dyown for ($($Tn),*) {
            type Owned = ($($Tn::Owned),*);
            fn own(&self) -> Self::Owned {
                let ($( $t_n ),*) = self;
                ($( $Tn::own($t_n) ),*)
            }
        }
    }
}
macro_rules! impl_dyeq {
    ($(($Tn:ident, $t_n:ident, $d_n:ident)),*) => {
        #[allow(unused_parens)]
        impl<$($Tn : DyeqItem),*> Dyeq for ($($Tn),*) {
            fn dyeq(&self, items: &[DynamicItem]) -> bool {
                let ($( $t_n ),*) = self;
                if let [$( $d_n ),*] = items {
                    true $(&& $t_n.dyeq_item($d_n))*
                } else {
                    false
                }
            }
        }
    };
}
all_tuples!(impl_dyeq, 1, 13, T, t, d);
all_tuples!(impl_dyown, 2, 13, T, t);
