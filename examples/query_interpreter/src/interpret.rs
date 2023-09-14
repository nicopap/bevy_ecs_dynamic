use std::any::TypeId;

use bevy::ecs::component::Components;
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::{ParsedPath, Reflect, TypeRegistryInternal as TypeRegistry};
use bevy_mod_dynamic_query::builder::NamedDynamicBuilder;
use bevy_mod_dynamic_query::pretty_print::DynShow;
use bevy_mod_dynamic_query::{DynamicItem, DynamicQuery};

use crate::eval_parser::{Expr, Operator as Op, Update};
use crate::query_parser::QueryStr;

fn get_ron(reg: &TypeRegistry, type_id: TypeId, input: &str) -> Option<Box<dyn Reflect>> {
    use ron::de::Deserializer as Ronzer;

    let registration = reg.get(type_id)?;
    let mut ron_de = Ronzer::from_str(input).ok()?;
    let de = TypedReflectDeserializer::new(registration, reg);
    de.deserialize(&mut ron_de).ok()
}
fn apply_number(target: &mut dyn Reflect, to_add: f32, to_mul: f32) -> Option<()> {
    macro_rules! try_mul_add {
        ([$($typ:ty),*]) => {
            $(try_mul_add!($typ));*
        };
        ($typ:ty) => {
            if let Some(x) = target.downcast_mut::<$typ>() {
                *x = *x * to_mul as $typ + to_add as $typ;
                return Some(());
            }
        };
    }
    try_mul_add!([f32, f64, usize, u8, u16, u32, u64, isize, i8, i16, i32, i64]);
    None
}
#[derive(Debug)]
pub enum Operator {
    Assign(String),
    Add(f32),
    Sub(f32),
    Mul(f32),
    Div(f32),
}
impl Operator {
    // TODO(err): use result
    fn apply(&self, target: &mut dyn Reflect, reg: &TypeRegistry) -> Option<()> {
        match self {
            Operator::Assign(value) => {
                let deserialized = get_ron(reg, target.type_id(), &value)?;
                target.apply(deserialized.as_ref());
                Some(())
            }
            &Operator::Add(value) => apply_number(target, value, 1.0),
            &Operator::Sub(value) => apply_number(target, -value, 1.0),
            &Operator::Mul(value) => apply_number(target, 0., value),
            &Operator::Div(value) => apply_number(target, 0., 1. / value),
        }
    }
    // TODO(err): use result
    fn from_update(update: Update) -> Option<Self> {
        match update.op {
            Op::Assign => Some(Self::Assign(update.rvalue.into())),
            Op::Add => Some(Self::Add(update.rvalue.parse().ok()?)),
            Op::Sub => Some(Self::Sub(update.rvalue.parse().ok()?)),
            Op::Mul => Some(Self::Mul(update.rvalue.parse().ok()?)),
            Op::Div => Some(Self::Div(update.rvalue.parse().ok()?)),
        }
    }
}

#[derive(Debug)]
struct ReflectUpdate {
    path: ParsedPath,
    action: Operator,
}

impl ReflectUpdate {
    fn from_expr(expr: Expr) -> Option<Self> {
        let path = ParsedPath::parse(dbg!(expr.path)).ok()?;
        let action = dbg!(Operator::from_update(expr.update))?;
        Some(Self { path, action })
    }
    fn apply(&self, target: &mut dyn Reflect, reg: &TypeRegistry) -> Option<()> {
        let target = self.path.reflect_element_mut(target).ok()?;
        self.action.apply(target, reg)
    }
}
#[derive(Resource, Debug, Default)]
pub struct Interpreter {
    eval: Vec<ReflectUpdate>,
    query: Option<DynamicQuery>,
    submit: bool,
}
impl Interpreter {
    // TODO(err): use result
    pub fn set_update(&mut self, update: Vec<Expr>) -> Option<()> {
        self.eval.clear();
        self.eval.extend(
            update
                .into_iter()
                .map(|t| ReflectUpdate::from_expr(t).unwrap()),
        );
        Some(())
    }
    pub fn set_query(
        &mut self,
        query: QueryStr,
        reg: &TypeRegistry,
        comps: &Components,
    ) -> Option<()> {
        let mut from_names = NamedDynamicBuilder::new(reg, comps);
        for fetch in &query.fetches {
            fetch.build(&mut from_names);
        }
        for conjunction in &query.filters {
            conjunction.build(&mut from_names);
        }
        self.query = Some(from_names.build()?);
        self.submit = true;
        Some(())
    }
    pub fn interpret(&self, world: &mut World, reg: &TypeRegistry) {
        let mut state = self.query.as_ref().unwrap().state(world);
        debug!("{self:?}");
        for (i, mut items) in state.iter_mut(world).enumerate() {
            info!("---> ({i}) {:?}", DynShow::new(&items));

            let items = items.iter_mut().filter_map(|item| match item {
                DynamicItem::Entity(_) => None,
                DynamicItem::Read(_) => None,
                DynamicItem::Mut(value) => Some(value),
                DynamicItem::OptionRead(_) => None,
                DynamicItem::OptionMut(value) => value.as_mut(),
            });
            let zipped = items.zip(self.eval.iter().cycle());

            for (value, update) in zipped {
                info!("(i) updating: {update:?}");
                update.apply(*value, reg);
            }
        }
    }
}
pub fn interpret_submitted_query(world: &mut World) {
    world.resource_scope(|world, mut interpret: Mut<Interpreter>| {
        if !interpret.submit {
            return;
        }
        interpret.submit = false;
        let reg = world.resource::<AppTypeRegistry>().clone();
        interpret.interpret(world, &reg.read());
    });
}
