use std::num::{ParseFloatError, ParseIntError};

use crate::ast_node::AstNode;
use crate::j::J;
use chrono;
use chrono::Datelike;
use pest::error::{Error as PestError, ErrorVariant};
use pest::Span;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use polars::datatypes::{CategoricalOrdering, DataType as PolarsDataType, Float64Type, TimeUnit};
use polars::frame::DataFrame;
use polars::prelude::{Column, IndexOrder, NamedFrom};
use polars::series::Series;
use regex::RegexSet;

const UNIX_EPOCH_DAY: i32 = 719_163;

pub const NS_IN_DAY: i64 = 86_400_000_000_000;
pub const NS_IN_MS: i64 = 1_000_000;

#[derive(Parser)]
#[grammar = "jasmine.pest"]
pub struct JParser;

fn parse_binary_op(pair: Pair<Rule>, source_id: usize) -> Result<AstNode, PestError<Rule>> {
    match pair.as_rule() {
        Rule::BinaryOp => Ok(AstNode::Operator {
            op: pair.as_str().to_owned(),
            pos: pair.as_span().start(),
            source_id,
        }),
        Rule::BinaryId => Ok(AstNode::Operator {
            op: pair.as_str()[1..].to_owned(),
            pos: pair.as_span().start(),
            source_id,
        }),
        _ => Err(raise_error(
            format!("Unexpected binary op/function: {}", pair.as_str()),
            pair.as_span(),
        )),
    }
}

fn parse_exp(pair: Pair<Rule>, source_id: usize) -> Result<AstNode, PestError<Rule>> {
    let rule = pair.as_rule();
    match rule {
        Rule::Exp => parse_exp(pair.into_inner().next().unwrap(), source_id),
        Rule::UnaryExp | Rule::UnarySqlExp => {
            let mut pair = pair.into_inner();
            let unary = pair.next().unwrap();
            let exp = pair.next().unwrap();
            let exp = parse_exp(exp, source_id)?;
            Ok(AstNode::UnaryExp {
                f: Box::new(parse_exp(unary, source_id)?),
                exp: Box::new(exp),
            })
        }
        Rule::BinaryExp | Rule::BinarySqlExp => {
            let mut pair = pair.into_inner();
            let lhs_pair = pair.next().unwrap();
            let lhs = parse_exp(lhs_pair, source_id)?;
            let binary_exp = pair.next().unwrap();
            let rhs_pair = pair.next().unwrap();
            let rhs = parse_exp(rhs_pair, source_id)?;
            Ok(AstNode::BinaryExp {
                f: Box::new(parse_binary_op(binary_exp, source_id)?),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            })
        }
        Rule::Integer
        | Rule::Boolean
        | Rule::Decimal
        | Rule::Date
        | Rule::Time
        | Rule::Datetime
        | Rule::Timestamp
        | Rule::Duration
        | Rule::String
        | Rule::None => parse_j(pair),
        Rule::Series => parse_series(pair),
        Rule::AssignmentExp => {
            let mut pairs = pair.into_inner();
            let id = pairs.next().unwrap();
            if id.as_rule() == Rule::FnCall {
                let mut fn_call = id.into_inner();
                let id = fn_call.next().unwrap().as_str();
                let mut indices: Vec<AstNode> = Vec::with_capacity(fn_call.len() - 1);
                for arg in fn_call {
                    indices.push(parse_exp(arg.into_inner().next().unwrap(), source_id)?)
                }
                let exp = parse_exp(pairs.next().unwrap(), source_id)?;
                Ok(AstNode::IndexAssignmentExp {
                    id: id.to_owned(),
                    indices,
                    exp: Box::new(exp),
                })
            } else {
                let exp = pairs.next().unwrap();
                let exp = parse_exp(exp, source_id)?;
                Ok(AstNode::AssignmentExp {
                    id: id.as_str().to_owned(),
                    exp: Box::new(exp),
                })
            }
        }
        Rule::Id => Ok(AstNode::Id {
            id: pair.as_str().to_owned(),
            pos: pair.as_span().start(),
            source_id: source_id,
        }),
        Rule::Fn => {
            let fn_body = pair.as_str();
            let fn_span = pair.as_span();
            let mut pairs = pair.into_inner();
            let pair = pairs.next().unwrap();
            let mut inner = pair.into_inner();
            let mut params: Vec<String> = Vec::with_capacity(inner.len());
            while let Some(pair) = inner.next() {
                params.push(pair.as_str().to_owned())
            }
            let mut nodes = Vec::with_capacity(pairs.len() - 1);
            for pair in pairs {
                nodes.push(parse_exp(pair, source_id)?)
            }
            Ok(AstNode::Fn {
                f: nodes,
                fn_body: fn_body.to_owned(),
                arg_num: params.len(),
                arg_names: params,
                args: Vec::new(),
                pos: fn_span.start(),
                source_id: source_id,
            })
        }
        Rule::FnCall => {
            let span = pair.as_span().start();
            let mut pairs = pair.into_inner();
            let f = parse_exp(pairs.next().unwrap(), source_id)?;
            let mut args = Vec::with_capacity(pairs.len() - 1);
            for pair in pairs {
                args.push(parse_exp(pair.into_inner().next().unwrap(), source_id)?)
            }
            // if f is eval, and first args is J::String, parse J::string
            Ok(AstNode::FnCall {
                f: Box::new(f),
                args,
            })
        }
        Rule::IfExp => {
            let mut pairs = pair.into_inner();
            let cond = parse_exp(pairs.next().unwrap(), source_id)?;
            let mut nodes = Vec::new();
            for pair in pairs.next().unwrap().into_inner() {
                let rule = pair.as_rule();
                nodes.push(parse_exp(pair, source_id)?);
                if rule == Rule::ReturnExp {
                    break;
                }
            }
            Ok(AstNode::If {
                cond: Box::new(cond),
                nodes,
            })
        }
        Rule::WhileExp => {
            let mut pairs = pair.into_inner();
            let cond = parse_exp(pairs.next().unwrap(), source_id)?;
            let mut nodes = Vec::new();
            for pair in pairs.next().unwrap().into_inner() {
                let rule = pair.as_rule();
                nodes.push(parse_exp(pair, source_id)?);
                if rule == Rule::ReturnExp {
                    break;
                }
            }
            Ok(AstNode::While {
                cond: Box::new(cond),
                nodes,
            })
        }
        Rule::TryExp => {
            let mut pairs = pair.into_inner();
            let mut tries = Vec::new();
            let mut catches = Vec::new();
            for pair in pairs.next().unwrap().into_inner() {
                tries.push(parse_exp(pair, source_id)?);
            }
            for pair in pairs.next().unwrap().into_inner() {
                catches.push(parse_exp(pair, source_id)?);
            }
            Ok(AstNode::Try { tries, catches })
        }
        Rule::ReturnExp => {
            let node = parse_exp(pair.into_inner().next().unwrap(), source_id)?;
            Ok(AstNode::Return(Box::new(node)))
        }
        Rule::RaiseExp => {
            let node = parse_exp(pair.into_inner().next().unwrap(), source_id)?;
            Ok(AstNode::Raise(Box::new(node)))
        }
        Rule::Skip => Ok(AstNode::Skip),
        Rule::Dataframe => {
            let span = pair.as_span();
            let cols = pair.into_inner();
            let mut col_exps: Vec<AstNode> = Vec::with_capacity(cols.len());
            let mut all_series = true;
            for (i, col_exp) in cols.enumerate() {
                let name: String;
                let exp: AstNode;
                let node = col_exp.into_inner().next().unwrap();
                if node.as_rule() == Rule::RenameColExp {
                    let mut nodes = node.into_inner();
                    name = nodes.next().unwrap().as_str().to_owned();
                    exp = parse_exp(nodes.next().unwrap(), source_id)?;
                } else {
                    name = format!("col{:02}", i);
                    exp = parse_exp(node, source_id)?
                }
                if let AstNode::J(j) = exp {
                    if let J::Series(mut s) = j {
                        s.rename(name.into());
                        col_exps.push(AstNode::J(J::Series(s)));
                    } else {
                        let mut s = j
                            .into_series()
                            .map_err(|e| raise_error(e.to_string(), span))?;
                        s.rename(name.into());
                        col_exps.push(AstNode::J(J::Series(s)));
                    }
                } else if let AstNode::Id(name) = &exp {
                    col_exps.push(AstNode::SeriesExp {
                        name: name.to_owned(),
                        exp: Box::new(exp),
                    });
                    all_series = false;
                } else {
                    col_exps.push(AstNode::SeriesExp {
                        name,
                        exp: Box::new(exp),
                    });
                    all_series = false;
                }
            }
            if all_series {
                let series: Vec<Column> = col_exps
                    .into_iter()
                    .map(|node| node.as_j().unwrap().series().unwrap().clone().into())
                    .collect();
                let df = match DataFrame::new(series) {
                    Ok(df) => df,
                    Err(e) => return Err(raise_error(e.to_string(), span)),
                };
                Ok(AstNode::J(J::DataFrame(df)))
            } else {
                Ok(AstNode::Dataframe(col_exps))
            }
        }
        Rule::Matrix => {
            let span = pair.as_span();
            let cols = pair.into_inner();
            let mut exps: Vec<AstNode> = Vec::with_capacity(cols.len());
            let mut all_series = true;
            for (i, col_exp) in cols.enumerate() {
                let col_name: String;
                let exp: AstNode;
                let node = col_exp.into_inner().next().unwrap();
                let node_span = node.as_span();
                col_name = format!("col{:02}", i);
                exp = parse_exp(node, source_id)?;
                if let AstNode::J(j) = exp {
                    let type_name = j.get_type_name();
                    if let J::Series(mut s) = j {
                        if !(s.dtype().is_numeric() || s.dtype().is_bool()) {
                            return Err(raise_error(
                                format!("Requires numeric data type, got '{}'", type_name),
                                node_span,
                            ));
                        }
                        s.rename(col_name.into());
                        exps.push(AstNode::J(J::Series(s)));
                    } else {
                        if !(j.is_numeric() || j.is_bool()) {
                            return Err(raise_error(
                                format!("Requires numeric data type, got '{}'", type_name),
                                node_span,
                            ));
                        }
                        let mut s = j.into_series().unwrap();
                        s.rename(col_name.into());
                        exps.push(AstNode::J(J::Series(s)));
                    }
                } else {
                    exps.push(AstNode::SeriesExp {
                        name: col_name,
                        exp: Box::new(exp),
                    });
                    all_series = false;
                }
            }
            if all_series {
                let cols: Vec<Column> = exps
                    .into_iter()
                    .map(|node| node.as_j().unwrap().series().unwrap().clone().into())
                    .collect();
                let df = match DataFrame::new(cols) {
                    Ok(df) => df,
                    Err(e) => return Err(raise_error(e.to_string(), span)),
                };
                let matrix = df
                    .to_ndarray::<Float64Type>(IndexOrder::C)
                    .map_err(|e| raise_error(e.to_string(), span))?;
                Ok(AstNode::J(J::Matrix(matrix.reversed_axes().to_shared())))
            } else {
                Ok(AstNode::Matrix(exps))
            }
        }
        Rule::SqlExp => parse_sql(pair, source_id),
        Rule::BracketExp | Rule::BracketSqlExp => {
            Ok(parse_exp(pair.into_inner().next().unwrap(), source_id)?)
        }
        Rule::List => {
            let pairs = pair.into_inner();
            let mut list = Vec::with_capacity(pairs.len());
            for pair in pairs {
                list.push(parse_list(pair, source_id)?)
            }
            Ok(AstNode::List(list))
        }
        Rule::Dict => {
            let pairs = pair.into_inner();
            let mut keys: Vec<String> = Vec::with_capacity(pairs.len());
            let mut values: Vec<AstNode> = Vec::with_capacity(pairs.len());
            for pair in pairs {
                let mut kv = pair.into_inner();
                keys.push(kv.next().unwrap().as_str().to_owned());
                values.push(parse_exp(kv.next().unwrap(), source_id)?)
            }
            Ok(AstNode::Dict { keys, values })
        }
        unexpected_exp => Err(raise_error(
            format!("Unexpected expression: {:?}", unexpected_exp),
            pair.as_span(),
        )),
    }
}

fn parse_list(pair: Pair<Rule>, source_id: usize) -> Result<AstNode, PestError<Rule>> {
    match pair.as_rule() {
        Rule::BinaryOp => Ok(AstNode::Operator {
            op: pair.as_str().to_owned(),
            pos: pair.as_span().start(),
            source_id: source_id,
        }),
        Rule::Exp => parse_exp(pair, source_id),
        _ => Err(raise_error(
            format!("Unexpected list expression: {:?}", pair.as_str()),
            pair.as_span(),
        )),
    }
}

macro_rules! impl_parse_num {
    ($fn_name:ident, $ty_str:literal, $ty:ty, $ty_err:ty) => {
        fn $fn_name(pair: Pair<Rule>) -> Result<AstNode, PestError<Rule>> {
            let span = pair.as_span();
            match pair
                .into_inner()
                .into_iter()
                .map(|p| {
                    let s = p.as_str();
                    let s = if s.ends_with($ty_str) {
                        &s[..s.len() - $ty_str.len()]
                    } else {
                        s
                    };
                    if s.is_empty() || s == "none" {
                        return Ok(None);
                    } else {
                        return s.parse::<$ty>().map(|n| Some(n));
                    }
                })
                .collect::<Result<Vec<Option<$ty>>, $ty_err>>()
            {
                Ok(n) => Ok(AstNode::J(J::Series(Series::new("".into(), n)))),
                Err(e) => Err(raise_error(e.to_string(), span)),
            }
        }
    };
}

impl_parse_num!(parse_u8, "u8", u8, ParseIntError);
impl_parse_num!(parse_i8, "i8", i8, ParseIntError);
impl_parse_num!(parse_u16, "u16", u16, ParseIntError);
impl_parse_num!(parse_i16, "i16", i16, ParseIntError);
impl_parse_num!(parse_u32, "u32", u32, ParseIntError);
impl_parse_num!(parse_i32, "i32", i32, ParseIntError);
impl_parse_num!(parse_u64, "u64", u64, ParseIntError);
impl_parse_num!(parse_i64, "i64", i64, ParseIntError);
impl_parse_num!(parse_f32, "f32", f32, ParseFloatError);
impl_parse_num!(parse_f64, "f64", f64, ParseFloatError);

fn parse_series(pair: Pair<Rule>) -> Result<AstNode, PestError<Rule>> {
    let mut first_scalar = "";
    for scalar in pair.into_inner() {
        if !scalar.as_str().is_empty() || scalar.as_str() != "none" {
            first_scalar = scalar.as_str();
            break;
        }
    }

    let set = RegexSet::new(&[
        r"(true|false)",
        r"\d+u8",
        r"-?\d+i8",
        r"\d+u16",
        r"-?\d+i16",
        r"\d+u32",
        r"-?\d+i32",
        r"\d+u64",
        r"-?\d+(i64)?",
        r"-?\d*\.?\d*f32",
        r"-?\d*\.?\d*(f64)?",
        r"\d{4}-\d{2}-\d{2}",
        r"\d{2}:\d{2}:\d{2}\.\d{,9}",
        r"\d{4}-\d{2}-\d{2}T(\d{2}:\d{2}:\d{2}(\.\d{,3})?)?",
        r"\d{4}-\d{2}-\d{2}D(\d{2}:\d{2}:\d{2}(\.\d{,9})?)?",
        r"-?\d+D(\d{2}:\d{2}:\d{2}(\.\d{,9})?)?",
        r"-\d+(ns|s|m|h)",
        r"`\S+",
        r#""[^"]*""#,
        r"none",
    ])
    .unwrap();

    let matches: Vec<_> = set.matches(&first_scalar).into_iter().collect();
    let first_match = matches.first().copied().unwrap_or(set.len());

    match first_match {
        0 => {
            let mut bools = Vec::with_capacity(pair.into_inner().len());
            for bool in pair.into_inner() {
                match bool.as_str() {
                    "true" => bools.push(Some(true)),
                    "false" => bools.push(Some(false)),
                    "none" | "" => bools.push(None),
                    _ => {
                        return Err(raise_error(
                            format!("unrecognized bool value {}", bool),
                            pair.as_span(),
                        ))
                    }
                }
            }
            let s = Series::new("".into(), bools);
            Ok(AstNode::J(J::Series(s)))
        }
        1 => parse_u8(pair),
        2 => parse_i8(pair),
        3 => parse_u16(pair),
        4 => parse_i16(pair),
        5 => parse_u32(pair),
        6 => parse_i32(pair),
        7 => parse_u64(pair),
        8 => parse_i64(pair),
        9 => parse_f32(pair),
        10 => parse_f64(pair),
        11 => {
            let dates = pair
                .as_str()
                .split_whitespace()
                .into_iter()
                .map(|s| J::parse_date(s).map_err(|e| raise_error(e.to_string(), pair.as_span())))
                .collect::<Result<Vec<J>, _>>()?
                .iter()
                .map(|j| j.to_i64().unwrap() as i32)
                .collect::<Vec<i32>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), dates)
                    .cast(&PolarsDataType::Date)
                    .map_err(|e| raise_error(e.to_string(), pair.as_span()))?,
            )))
        }
        12 => {
            let times = pair
                .as_str()
                .split_whitespace()
                .into_iter()
                .map(|s| J::parse_time(s).map_err(|e| raise_error(e.to_string(), pair.as_span())))
                .collect::<Result<Vec<J>, _>>()?
                .iter()
                .map(|j| j.to_i64().unwrap())
                .collect::<Vec<i64>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), times)
                    .cast(&PolarsDataType::Time)
                    .map_err(|e| raise_error(e.to_string(), pair.as_span()))?,
            )))
        }
        13 => {
            let datetimes = pair
                .as_str()
                .split_whitespace()
                .into_iter()
                .map(|s| {
                    J::parse_datetime(s).map_err(|e| raise_error(e.to_string(), pair.as_span()))
                })
                .collect::<Result<Vec<J>, _>>()?
                .iter()
                .map(|j| j.to_i64().unwrap())
                .collect::<Vec<i64>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), datetimes)
                    .cast(&PolarsDataType::Datetime(TimeUnit::Milliseconds, None))
                    .map_err(|e| raise_error(e.to_string(), pair.as_span()))?,
            )))
        }
        14 => {
            let timestamps = pair
                .as_str()
                .split_whitespace()
                .into_iter()
                .map(|s| {
                    J::parse_timestamp(s).map_err(|e| raise_error(e.to_string(), pair.as_span()))
                })
                .collect::<Result<Vec<J>, _>>()?
                .iter()
                .map(|j| j.to_i64().unwrap())
                .collect::<Vec<i64>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), timestamps)
                    .cast(&PolarsDataType::Datetime(TimeUnit::Nanoseconds, None))
                    .map_err(|e| raise_error(e.to_string(), pair.as_span()))?,
            )))
        }
        15 | 16 => {
            let times = pair
                .as_str()
                .split_whitespace()
                .into_iter()
                .map(|s| {
                    J::parse_duration(s).map_err(|e| raise_error(e.to_string(), pair.as_span()))
                })
                .collect::<Result<Vec<J>, _>>()?
                .iter()
                .map(|j| j.to_i64().unwrap())
                .collect::<Vec<i64>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), times)
                    .cast(&PolarsDataType::Duration(TimeUnit::Nanoseconds))
                    .map_err(|e| raise_error(e.to_string(), pair.as_span()))?,
            )))
        }

        Rule::Syms => {
            let syms = pair.as_str()[1..].split("`").collect::<Vec<_>>();
            Ok(AstNode::J(J::Series(
                Series::new("".into(), syms)
                    .cast(&PolarsDataType::Categorical(
                        None,
                        CategoricalOrdering::Lexical,
                    ))
                    .unwrap(),
            )))
        }
    }
}

fn parse_j(pair: Pair<Rule>) -> Result<AstNode, PestError<Rule>> {
    match pair.as_rule() {
        Rule::Boolean => Ok(AstNode::J(J::Boolean(pair.as_str() == "1b"))),
        Rule::Integer => match pair.as_str().parse::<i64>() {
            Ok(n) => Ok(AstNode::J(J::I64(n))),
            Err(e) => Err(raise_error(e.to_string(), pair.as_span())),
        },
        Rule::Decimal => match pair.as_str().parse::<f64>() {
            Ok(n) => Ok(AstNode::J(J::F64(n))),
            Err(e) => Err(raise_error(e.to_string(), pair.as_span())),
        },
        Rule::Date => {
            let j = parse_date(pair.as_str())
                .map_err(|e| raise_error(e.to_string(), pair.as_span()))
                .map(|j| J::Date(j))?;
            Ok(AstNode::J(j))
        }
        Rule::Time => {
            let j = parse_time(pair.as_str())
                .map_err(|e| raise_error(e.to_string(), pair.as_span()))
                .map(|j| J::Time(j))?;
            Ok(AstNode::J(j))
        }
        Rule::Datetime => {
            let j = parse_datetime(pair.as_str())
                .map_err(|e| raise_error(e.to_string(), pair.as_span()))
                .map(|j| J::Datetime(j))?;
            Ok(AstNode::J(j))
        }
        Rule::Timestamp => {
            let j = parse_timestamp(pair.as_str())
                .map_err(|e| raise_error(e.to_string(), pair.as_span()))
                .map(|j| J::Timestamp(j))?;
            Ok(AstNode::J(j))
        }
        Rule::Duration => {
            let j = parse_duration(pair.as_str())
                .map_err(|e| raise_error(e.to_string(), pair.as_span()))
                .map(|j| J::Duration(j))?;
            Ok(AstNode::J(j))
        }
        Rule::Enum => Ok(AstNode::J(J::Symbol(pair.as_str()[1..].to_string()))),
        Rule::String => {
            let str = pair.as_str();
            // Strip leading and ending quotes.
            let str = &str[1..str.len() - 1];
            // Escaped string quotes become single quotes here.
            Ok(AstNode::J(J::String(str.to_owned())))
        }
        unexpected_exp => Err(raise_error(
            format!("Unexpected j: {:?}", unexpected_exp),
            pair.as_span(),
        )),
    }
}

fn parse_sql(pair: Pair<Rule>, source_id: usize) -> Result<AstNode, PestError<Rule>> {
    let mut pairs = pair.into_inner();
    // select, update, exec, delete
    let op = get_built_in_fn(&pairs.next().unwrap())?;
    let mut op_exp: Vec<AstNode> = Vec::new();
    let mut by_exp: Vec<AstNode> = Vec::new();
    let mut from_exp: AstNode = AstNode::Skip;
    let mut where_exp: Vec<AstNode> = Vec::new();
    while let Some(some_pair) = pairs.next() {
        match some_pair.as_rule() {
            Rule::SelectExp | Rule::ColNames => {
                let op_pairs = some_pair.into_inner();
                for op_pair in op_pairs {
                    op_exp.push(parse_sql_col_exp(op_pair, source_id)?)
                }
            }
            Rule::ByExp => {
                let by_pairs = some_pair.into_inner();
                by_exp = Vec::with_capacity(by_pairs.len());
                for by_pair in by_pairs {
                    by_exp.push(parse_sql_col_exp(by_pair, source_id)?)
                }
            }
            Rule::FromExp | Rule::FromWithoutWhereExp => {
                from_exp = parse_exp(some_pair.into_inner().next().unwrap(), source_id)?
            }
            Rule::FilterExp => {
                let where_pairs = some_pair.into_inner();
                where_exp = Vec::with_capacity(where_pairs.len());
                for where_pair in where_pairs {
                    where_exp.push(parse_exp(where_pair, source_id)?)
                }
            }
            unexpected_exp => {
                return Err(raise_error(
                    format!("Unexpected sql: {:?}", unexpected_exp),
                    some_pair.as_span(),
                ))
            }
        }
    }
    Ok(AstNode::Sql {
        op,
        op_exp,
        group_exp: by_exp,
        from_exp: Box::new(from_exp),
        filter_exp: where_exp,
    })
}

fn parse_sql_col_exp(pair: Pair<Rule>, source_id: usize) -> Result<AstNode, PestError<Rule>> {
    match pair.as_rule() {
        Rule::ColExp => parse_sql_col_exp(pair.into_inner().next().unwrap(), source_id),
        Rule::RenameColExp => {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str();
            let exp = parse_exp(pairs.next().unwrap(), source_id)?;
            Ok(AstNode::SeriesExp {
                name: name.to_owned(),
                exp: Box::new(exp),
            })
        }
        Rule::ColName => Ok(AstNode::Id(
            pair.into_inner().next().unwrap().as_str().to_owned(),
        )),
        _ => parse_exp(pair, source_id),
    }
}

fn raise_error(msg: String, span: Span) -> PestError<Rule> {
    PestError::new_from_span(ErrorVariant::CustomError { message: msg }, span)
}

pub fn parse(source: &str, source_id: usize) -> Result<Vec<AstNode>, PestError<Rule>> {
    let mut ast = vec![];
    let pairs = JParser::parse(Rule::Program, source)?;
    for pair in pairs {
        if let Rule::Exp = pair.as_rule() {
            ast.push(parse_exp(pair, source_id)?);
        }
    }
    Ok(ast)
}

pub fn parse_date(date: &str) -> Result<i32, String> {
    match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(d) => Ok(d.num_days_from_ce() - UNIX_EPOCH_DAY),
        Err(_) => Err(format!("Not a valid date, {}", date)),
    }
}

pub fn parse_time(time: &str) -> Result<i64, String> {
    let err = || format!("Not a valid time, {}", time);
    let mut nano = "";
    let time = if time.len() > 8 {
        let v: Vec<&str> = time.split(".").collect();
        nano = v[1];
        v[0]
    } else {
        time
    };
    let v: Vec<&str> = time.split(":").collect();
    let hh = v[0].parse::<i64>().map_err(|_| err())?;
    if hh > 23 {
        return Err(err());
    }
    let mm = v[1].parse::<i64>().map_err(|_| err())?;
    if mm > 59 {
        return Err(err());
    }
    let ss = v[2].parse::<i64>().map_err(|_| err())?;
    if ss > 59 {
        return Err(err());
    }
    let nano = format!("{:0<9}", nano);
    let nano = nano.parse::<i64>().map_err(|_| err())?;
    if nano > 999_999_999 {
        return Err(err());
    }
    Ok((hh * 3600 + mm * 60 + ss) * 1000_000_000 + nano)
}

pub fn parse_duration(duration: &str) -> Result<i64, String> {
    let err = || format!("Not a valid duration, {}", duration);
    if duration.contains("D") {
        let v: Vec<&str> = duration.split("D").collect();
        let time = v[1];
        let day = v[0].parse::<i64>().map_err(|_| err())?;
        let nano = if time == "" {
            0
        } else {
            parse_time(time).map_err(|_| err())?
        };
        Ok(day * NS_IN_DAY + nano)
    } else if duration.ends_with("ns") {
        duration[..duration.len() - 2]
            .parse::<i64>()
            .map_err(|_| err())
    } else if duration.ends_with("s") {
        duration[..duration.len() - 2]
            .parse::<i64>()
            .map_err(|_| err())
            .map(|u| u * 1000_000_000)
    } else if duration.ends_with("m") {
        duration[..duration.len() - 2]
            .parse::<i64>()
            .map_err(|_| err())
            .map(|u| u * 60_000_000_000)
    } else if duration.ends_with("h") {
        duration[..duration.len() - 2]
            .parse::<i64>()
            .map_err(|_| err())
            .map(|u| u * 3_600_000_000_000)
    } else {
        return Err(err());
    }
}

pub fn parse_datetime(datetime: &str) -> Result<i64, String> {
    match chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%.f") {
        Ok(d) => Ok(d.and_utc().timestamp_millis()),
        Err(_) => Err(format!("Not a valid datetime, {}", datetime)),
    }
}

pub fn parse_timestamp(datetime: &str) -> Result<i64, String> {
    match chrono::NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dD%H:%M:%S%.f") {
        Ok(d) => Ok(d.and_utc().timestamp_nanos_opt().unwrap_or(0)),
        Err(_) => Err(format!("Not a valid datetime, {}", datetime)),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{parse_duration, parse_time};

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("23:59:59").unwrap(), 86399000000000);
        assert_eq!(parse_time("07:59:59").unwrap(), 28799000000000);
        assert_eq!(parse_time("23:59:59.").unwrap(), 86399000000000);
        assert_eq!(parse_time("23:59:59.123456789").unwrap(), 86399123456789);
        assert_eq!(parse_time("23:59:59.123").unwrap(), 86399123000000);
        assert_eq!(parse_time("23:59:59.000123").unwrap(), 86399000123000);
        assert!(parse_time("24:59:59.123456789").is_err())
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("0D23:59:59").unwrap(), 86399000000000);
        assert_eq!(parse_duration("1D23:59:59").unwrap(), 172799000000000);
        assert_eq!(parse_duration("100D23:59:59").unwrap(), 8726399000000000);
        assert!(parse_duration("100D23:60:59.123456789").is_err())
    }
}