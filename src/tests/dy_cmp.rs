//! A wrapper type to compare any tuple to &[DynamicItem]

use bevy_ecs::{all_tuples, prelude::Entity};
use bevy_reflect::Reflect;

use crate::DynamicItem;

pub trait Dyeq {
    fn dyeq(&self, items: &[DynamicItem]) -> bool;
}
pub trait DyeqItem {
    fn dyeq_item(&self, item: &DynamicItem) -> bool;
}
impl<T: PartialEq + Reflect> DyeqItem for &'_ T {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Read(r) if r.downcast_ref() == Some(*self))
    }
}
impl<T: PartialEq + Reflect> DyeqItem for &'_ mut T {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Mut(r) if r.downcast_ref() == Some(*self))
    }
}
impl DyeqItem for Entity {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        matches!(item, DynamicItem::Entity(e) if e == self)
    }
}
// TODO(BUG): This is wrong for when None.
impl<T: PartialEq + Reflect> DyeqItem for Option<&'_ T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        use DynamicItem::OptionRead;
        matches!(item, OptionRead(r) if r.map(<dyn Reflect>::downcast_ref) == self.map(Some))
    }
}
impl<T: PartialEq + Reflect> DyeqItem for Option<&'_ mut T> {
    fn dyeq_item(&self, item: &DynamicItem) -> bool {
        use DynamicItem::OptionMut;
        let opt: Option<Option<&T>> = self.as_deref().map(Some);
        matches!(item, OptionMut(r) if r.as_ref().map(|r| r.downcast_ref()) == opt)
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
