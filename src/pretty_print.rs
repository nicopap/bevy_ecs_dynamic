//! Nice formatting for `[DynamicItem]`

use std::{fmt, marker::PhantomData};

use bevy_ecs::{all_tuples, prelude::Entity};
use bevy_reflect::Reflect;

use crate::DynamicItem;

pub struct DynShow<'w, T: AsRef<[DynamicItem<'w>]>>(T, PhantomData<&'w ()>);
pub struct DynShowT<'a, T: ShowReflect>(pub &'a T);

impl<'w, T: AsRef<[DynamicItem<'w>]>> DynShow<'w, T> {
    pub fn new(inner: T) -> Self {
        Self(inner, PhantomData)
    }
}

struct DynItem<'a, 'b>(&'a DynamicItem<'b>);

impl fmt::Debug for DynItem<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            DynamicItem::Entity(e) => write!(f, "entity{e:?}"),
            DynamicItem::Read(value) => {
                f.write_str("&")?;
                value.debug(f)
            }
            DynamicItem::Mut(value) => {
                f.write_str("&mut ")?;
                value.debug(f)
            }
            DynamicItem::OptionRead(None) => f.write_str("None"),
            DynamicItem::OptionMut(None) => f.write_str("None"),
            DynamicItem::OptionRead(Some(value)) => {
                f.write_str("Some(&")?;
                value.debug(f)?;
                f.write_str(")")
            }
            DynamicItem::OptionMut(Some(value)) => {
                f.write_str("Some(&mut ")?;
                value.debug(f)?;
                f.write_str(")")
            }
        }
    }
}

impl<'w, T: AsRef<[DynamicItem<'w>]>> fmt::Debug for DynShow<'w, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.0.as_ref();
        if items.len() == 0 {
            return f.write_str("()");
        }
        if items.len() == 1 {
            return DynItem(&items[0]).fmt(f);
        }
        let mut tuple = f.debug_tuple("");
        for item in items {
            tuple.field(&DynItem(item));
        }
        tuple.finish()
    }
}
impl<T: ShowReflect> fmt::Debug for DynShowT<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.show(f)
    }
}

pub trait ShowReflect {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result;
}
impl<T: Reflect> ShowReflect for &'_ T {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("&")?;
        self.debug(f)
    }
}
impl<T: Reflect> ShowReflect for &'_ mut T {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("&mut ")?;
        self.debug(f)
    }
}
impl ShowReflect for Entity {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "entity{self:?}")
    }
}
impl<T: Reflect> ShowReflect for Option<&'_ T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = self {
            f.write_str("Some(&")?;
            value.debug(f)?;
            f.write_str(")")
        } else {
            f.write_str("None")
        }
    }
}
impl<T: Reflect> ShowReflect for Option<&'_ mut T> {
    fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(value) = self {
            f.write_str("Some(&mut ")?;
            value.debug(f)?;
            f.write_str(")")
        } else {
            f.write_str("None")
        }
    }
}

macro_rules! impl_show_reflect {
    ($(($Tn:ident, $t_n:ident)),*) => {
        #[allow(unused_parens)]
        impl<$($Tn : ShowReflect),*> ShowReflect for ($($Tn),*) {
            fn show(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let ($($t_n),*) = self;

                f.debug_tuple("")
                 $( .field(&DynShowT($t_n)) )*
                    .finish()
            }
        }
    };
}
all_tuples!(impl_show_reflect, 2, 13, T, t);
