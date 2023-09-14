use winnow::combinator::{alt, delimited, preceded, repeat, separated0, separated1};
use winnow::{ascii::alphanumeric1, ascii::multispace0, token::tag, PResult, Parser};

macro_rules! ws {
    ($parser:expr) => {
        delimited(multispace0, $parser, multispace0)
    };
}

#[derive(Debug)]
pub struct Expr<'a> {
    pub path: &'a str,
    pub update: Update<'a>,
}
#[derive(Debug)]
pub struct Update<'a> {
    pub op: Operator,
    pub rvalue: &'a str,
}
#[derive(Debug)]
pub enum Operator {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
}
pub fn expressions<'i>(input: &mut &'i str) -> PResult<Vec<Expr<'i>>> {
    separated0(expression, ",").parse_next(input)
}
fn expression<'i>(input: &mut &'i str) -> PResult<Expr<'i>> {
    (path, update)
        .map(|(path, update)| Expr { path, update })
        .parse_next(input)
}
fn path<'i>(input: &mut &'i str) -> PResult<&'i str> {
    let path_element = alt((preceded(".", ident), delimited("[", ident, "]"))).void();
    repeat::<_, _, (), _, _>(1.., path_element)
        .recognize()
        .parse_next(input)
}
fn update<'i>(input: &mut &'i str) -> PResult<Update<'i>> {
    let op = alt((
        tag("+=").map(|_| Operator::Add),
        tag("-=").map(|_| Operator::Sub),
        tag("*=").map(|_| Operator::Mul),
        tag("/=").map(|_| Operator::Div),
        tag("=").map(|_| Operator::Assign),
    ));
    (ws!(op), ident)
        .map(|(op, rvalue)| Update { op, rvalue })
        .parse_next(input)
}
fn ident<'i>(input: &mut &'i str) -> PResult<&'i str> {
    separated1::<_, _, (), _, _, _, _>(alphanumeric1, "_")
        .recognize()
        .parse_next(input)
}
