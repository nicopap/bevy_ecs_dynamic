use bevy_mod_dynamic_query::builder::{NamedDynamicBuilder, NamedOrBuilder};
use winnow::{
    ascii::{alpha1, alphanumeric1, multispace0},
    combinator::{alt, delimited, opt, preceded, repeat, separated1},
    PResult, Parser,
};

#[derive(Clone, Copy, Debug)]
pub enum AndFilterStr<'a> {
    With(&'a str),
    Without(&'a str),
    Changed(&'a str),
    Added(&'a str),
}
impl AndFilterStr<'_> {
    fn builder(&self, builder: &mut NamedOrBuilder) {
        match self {
            AndFilterStr::With(name) => builder.with(name),
            AndFilterStr::Without(name) => builder.without(name),
            AndFilterStr::Changed(name) => builder.changed(name),
            AndFilterStr::Added(name) => builder.added(name),
        };
    }
}
#[derive(Clone, Debug)]
pub struct AndFiltersStr<'a>(pub Vec<AndFilterStr<'a>>);
impl AndFiltersStr<'_> {
    pub fn build(&self, builder: &mut NamedDynamicBuilder) {
        builder.or(|builder| {
            for filter in &self.0 {
                filter.builder(builder);
            }
            builder
        });
    }
}

#[derive(Clone, Debug)]
pub enum FetchStr<'a> {
    Read(&'a str),
    Mut(&'a str),
    OptionRead(&'a str),
    OptionMut(&'a str),
    Entity,
}
impl FetchStr<'_> {
    pub fn build(&self, builder: &mut NamedDynamicBuilder) {
        match self {
            FetchStr::Read(name) => builder.component(name),
            FetchStr::Mut(name) => builder.component_mut(name),
            FetchStr::OptionRead(name) => builder.optional(name),
            FetchStr::OptionMut(name) => builder.optional_mut(name),
            FetchStr::Entity => builder.entity(),
        };
    }
}

#[derive(Debug)]
pub struct QueryStr<'a> {
    pub fetches: Vec<FetchStr<'a>>,
    pub filters: Vec<AndFiltersStr<'a>>,
}

macro_rules! ws {
    ($parser:expr) => {
        delimited(multispace0, $parser, multispace0)
    };
}

pub fn query<'i>(input: &mut &'i str) -> PResult<QueryStr<'i>> {
    let conjunction = || repeat(1.., ws!(filter)).map(AndFiltersStr);
    let or_filters = || separated1(ws!(conjunction()), "|");
    let fetches = repeat(1.., ws!(fetch));

    (fetches, opt(preceded(",", ws!(or_filters()))))
        .map(|(fetches, filters)| QueryStr { fetches, filters: filters.unwrap_or(Vec::new()) })
        .parse_next(input)
}
fn fetch<'i>(input: &mut &'i str) -> PResult<FetchStr<'i>> {
    alt((
        "Entity".map(|_| FetchStr::Entity),
        preceded("mut ", component).map(FetchStr::Mut),
        preceded("?mut ", component).map(FetchStr::OptionMut),
        preceded("?", component).map(FetchStr::OptionRead),
        component.map(FetchStr::Read),
    ))
    .parse_next(input)
}
fn filter<'i>(input: &mut &'i str) -> PResult<AndFilterStr<'i>> {
    alt((
        preceded("!", component).map(AndFilterStr::Without),
        preceded("+", component).map(AndFilterStr::Added),
        preceded(">", component).map(AndFilterStr::Changed),
        component.map(AndFilterStr::With),
    ))
    .parse_next(input)
}
fn component<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let repeat = repeat::<_, _, (), _, _>;
    (alt((alpha1, "_")), repeat(.., alt((alphanumeric1, "_"))))
        .recognize()
        .parse_next(input)
}
// pub fn query<'i>(input: &mut &'i str) -> PResult<QueryStr<'i>> {
//     delimited(
//         "Query<",
//         (ws!(fetches), opt(preceded(",", ws!(or_filters)))),
//         ">",
//     )
//     .map(|(fetches, filters)| QueryStr { fetches, filters: filters.unwrap_or(Vec::new()) })
//     .parse_next(input)
// }
// fn fetches<'i>(input: &mut &'i str) -> PResult<Vec<FetchStr<'i>>> {
//     alt((
//         delimited("(", separated1(ws!(fetch), ","), ")"),
//         ws!(fetch).map(|f| vec![f]),
//     ))
//     .parse_next(input)
// }
// fn fetch<'i>(input: &mut &'i str) -> PResult<FetchStr<'i>> {
//     alt((
//         preceded("&", component).map(FetchStr::Read),
//         preceded("&mut ", component).map(FetchStr::Mut),
//         delimited("Option<&", component, ">").map(FetchStr::OptionRead),
//         delimited("Option<&mut ", component, ">").map(FetchStr::OptionMut),
//         "Entity".map(|_| FetchStr::Entity),
//     ))
//     .parse_next(input)
// }
// fn or_filters<'i>(input: &mut &'i str) -> PResult<Vec<AndFiltersStr<'i>>> {
//     delimited("Or<(", separated1(ws!(conjunction), ","), ")>").parse_next(input)
// }
// fn conjunction<'i>(input: &mut &'i str) -> PResult<AndFiltersStr<'i>> {
//     alt((
//         delimited("(", separated1(ws!(filter), ","), ")"),
//         filter.map(|f| vec![f]),
//     ))
//     .map(AndFiltersStr)
//     .parse_next(input)
// }
// fn filter<'i>(input: &mut &'i str) -> PResult<AndFilterStr<'i>> {
//     alt((
//         delimited("With<", component, ">").map(AndFilterStr::With),
//         delimited("Without<", component, ">").map(AndFilterStr::Without),
//         delimited("Added<", component, ">").map(AndFilterStr::Added),
//         delimited("Changed<", component, ">").map(AndFilterStr::Changed),
//     ))
//     .parse_next(input)
// }
