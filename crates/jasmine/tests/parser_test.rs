use jasmine::{JParser, Rule};
use pest::Parser;

use crate::util::pretty_format_rules;

#[path = "./util.rs"]
mod util;

#[test]
fn parse_comments() {
    let code = "/*
    block of comment
*/

\"string❤️\"; // comment

// comment
\"string\";

/* */
    ";
    let pairs = JParser::parse(Rule::Program, code).unwrap();
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            // "COMMENT -> blockComment",
            "Exp -> String",
            // "COMMENT -> lineComment",
            // "COMMENT -> lineComment",
            "Exp -> String",
            // "COMMENT -> blockComment",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case00() {
    let code = "total = sum [1.0,2.0]*3";
    let pairs = JParser::parse(Rule::Program, code).unwrap();
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> UnaryExp",
            "       -> Id",
            "       -> Exp -> BinaryExp",
            "           -> Series",
            "             -> Unknown",
            "             -> Unknown",
            "           -> BinaryOp",
            "           -> Exp -> Integer",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case01() {
    let code = "
    f = fn(x,y,z){x + y * z};
    r = f(1, 2, 3);
    g = f(1, , 9);
    h = fn(){9};
    g 3
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Fn",
            "       -> Params",
            "         -> Id",
            "         -> Id",
            "         -> Id",
            "       -> Exp -> BinaryExp",
            "           -> Id",
            "           -> BinaryOp",
            "           -> Exp -> BinaryExp",
            "               -> Id",
            "               -> BinaryOp",
            "               -> Exp -> Id",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> FnCall",
            "       -> GlobalId",
            "       -> Arg -> Exp -> Integer",
            "       -> Arg -> Exp -> Integer",
            "       -> Arg -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> FnCall",
            "       -> GlobalId",
            "       -> Arg -> Exp -> Integer",
            "       -> Arg -> Skip",
            "       -> Arg -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Fn",
            "       -> Params",
            "       -> Exp -> Integer",
            "Exp -> UnaryExp",
            "   -> Id",
            "   -> Exp -> Integer",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case02() {
    let code = "
    qty = [7i16,8,9];
    df0 = df[sym = [`a,`b,`b], col1 = [1, 2, 3], col2 = [1.0, 2.0, 3.0], [4, 5, 6], qty];
    df0 = from t filter {sym==`a`} group {sym} select { sum col1+col2, newCol= col2 };
    count df0
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Series",
            "       -> Unknown",
            "       -> Unknown",
            "       -> Unknown",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Dataframe",
            "       -> SeriesExp -> RenameSeriesExp",
            "           -> SeriesName",
            "           -> Series",
            "             -> Unknown",
            "             -> Unknown",
            "             -> Unknown",
            "       -> SeriesExp -> RenameSeriesExp",
            "           -> SeriesName",
            "           -> Series",
            "             -> Unknown",
            "             -> Unknown",
            "             -> Unknown",
            "       -> SeriesExp -> RenameSeriesExp",
            "           -> SeriesName",
            "           -> Series",
            "             -> Unknown",
            "             -> Unknown",
            "             -> Unknown",
            "       -> SeriesExp -> Series",
            "           -> Unknown",
            "           -> Unknown",
            "           -> Unknown",
            "       -> SeriesExp -> Id",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> SqlExp",
            "       -> FromExp -> Id",
            "       -> FilterExp -> BinarySqlExp",
            "           -> Id",
            "           -> BinaryOp",
            "           -> Enum",
            "       -> GroupExp -> SeriesExp -> Id",
            "       -> SelectOp",
            "         -> SeriesExp -> UnarySqlExp",
            "             -> Id",
            "             -> BinarySqlExp",
            "               -> Id",
            "               -> BinaryOp",
            "               -> Id",
            "         -> SeriesExp -> RenameSeriesExp",
            "             -> SeriesName",
            "             -> Id",
            "Exp -> UnaryExp",
            "   -> Id",
            "   -> Exp -> Id",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case02_01() {
    let code = "
    from df filter {col!=1} sort {col1, -col2} take 10;
    from df delete {col1, col2, col3};
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> SqlExp",
            "   -> FromExp -> Id",
            "   -> FilterExp -> BinarySqlExp",
            "       -> Id",
            "       -> BinaryOp",
            "       -> Integer",
            "   -> SortOp",
            "     -> SortName",
            "     -> SortName",
            "   -> TakeOp -> Exp -> Integer",
            "Exp -> SqlExp",
            "   -> FromExp -> Id",
            "   -> DeleteOp",
            "     -> SeriesName",
            "     -> SeriesName",
            "     -> SeriesName",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case03() {
    let code = "
    r1 = eval l[*, 9, 9];
    f = fn(x, y){ x - y};
    r2 = eval(l[`f`, 9, 1]);
    t = timeit(l[+, 1, 1], 1000);
    t
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> UnaryExp",
            "       -> Id",
            "       -> Exp -> List",
            "           -> BinaryOp",
            "           -> Exp -> Integer",
            "           -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Fn",
            "       -> Params",
            "         -> Id",
            "         -> Id",
            "       -> Exp -> BinaryExp",
            "           -> Id",
            "           -> BinaryOp",
            "           -> Exp -> Id",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> FnCall",
            "       -> GlobalId",
            "       -> Arg -> Exp -> List",
            "             -> Exp -> Enum",
            "             -> Exp -> Integer",
            "             -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> FnCall",
            "       -> GlobalId",
            "       -> Arg -> Exp -> List",
            "             -> BinaryOp",
            "             -> Exp -> Integer",
            "             -> Exp -> Integer",
            "       -> Arg -> Exp -> Integer",
            "Exp -> Id",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case04() {
    let code = "
    d = {a: 1, b: 2, c: 3,};
    d2 = {a: 4, b: 5, c: 6};
    d3 = d[[`a`,`b`,`c`], [7,8,9]];
    d(`d`) = 9;
    r1 = d2 (`c`);
    d3(`c`) + sum d([`a`,`d`])
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Dict",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Dict",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "       -> KeyValueExp",
            "         -> Id",
            "         -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Dict",
            "       -> Keys",
            "         -> Exp -> Enum",
            "         -> Exp -> Enum",
            "         -> Exp -> Enum",
            "       -> Values",
            "         -> Exp -> Integer",
            "         -> Exp -> Integer",
            "         -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> FnCall",
            "     -> GlobalId",
            "     -> Arg -> Exp -> Enum",
            "   -> Exp -> Integer",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> FnCall",
            "       -> GlobalId",
            "       -> Arg -> Exp -> Enum",
            "Exp -> BinaryExp",
            "   -> FnCall",
            "     -> GlobalId",
            "     -> Arg -> Exp -> Enum",
            "   -> BinaryOp",
            "   -> Exp -> UnaryExp",
            "       -> Id",
            "       -> Exp -> FnCall",
            "           -> GlobalId",
            "           -> Arg -> Exp -> Series",
            "                 -> Unknown",
            "                 -> Unknown",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case05() {
    let code = "
    f = fn(data){
        if(date>2020-01-01){
            raise error;
            return date;
            date= date + 1;
        };
        2020-01-01
    };;
    r1 = f 2024-04-01;
    r2 = f 2019-01-01;
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> Fn",
            "       -> Params -> Id",
            "       -> Exp -> IfExp",
            "           -> ConditionExp -> BinaryExp",
            "               -> Id",
            "               -> BinaryOp",
            "               -> Exp -> Date",
            "           -> Statements",
            "             -> Exp -> UnaryExp",
            "                 -> GlobalId",
            "                 -> Exp -> Id",
            "             -> Exp -> UnaryExp",
            "                 -> GlobalId",
            "                 -> Exp -> Id",
            "             -> Exp -> AssignmentExp",
            "                 -> Id",
            "                 -> Exp -> BinaryExp",
            "                     -> Id",
            "                     -> BinaryOp",
            "                     -> Exp -> Integer",
            "       -> Exp -> Date",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> UnaryExp",
            "       -> Id",
            "       -> Exp -> Date",
            "Exp -> AssignmentExp",
            "   -> Id",
            "   -> Exp -> UnaryExp",
            "       -> Id",
            "       -> Exp -> Date",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case06() {
    let code = "
    nest(fn(x){x++sum -2#x}, [1,1], 10)
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> FnCall",
            "   -> GlobalId",
            "   -> Arg -> Exp -> Fn",
            "         -> Params -> Id",
            "         -> Exp -> BinaryExp",
            "             -> Id",
            "             -> BinaryOp",
            "             -> Exp -> UnaryExp",
            "                 -> Id",
            "                 -> Exp -> BinaryExp",
            "                     -> Integer",
            "                     -> BinaryOp",
            "                     -> Exp -> Id",
            "   -> Arg -> Exp -> Series",
            "         -> Unknown",
            "         -> Unknown",
            "   -> Arg -> Exp -> Integer",
            "EOI",
            ""
        ],
        actual
    )
}

#[test]
fn parse_case07() {
    let code = "
    try {
        a = 1 + `a`;
    } catch(err) {
        err == \"type\";
    }
    ";
    let pairs = match JParser::parse(Rule::Program, code) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            panic!("failed to parse")
        }
    };
    let binding = pretty_format_rules(pairs);
    let actual: Vec<&str> = binding.split("\n").collect();
    assert_eq!(
        vec![
            "Exp -> TryExp",
            "   -> Statements -> Exp -> AssignmentExp",
            "         -> Id",
            "         -> Exp -> BinaryExp",
            "             -> Integer",
            "             -> BinaryOp",
            "             -> Exp -> Enum",
            "   -> Id",
            "   -> Statements -> Exp -> BinaryExp",
            "         -> Id",
            "         -> BinaryOp",
            "         -> Exp -> String",
            "EOI",
            ""
        ],
        actual
    )
}
