use crate::errors::{JError, JResult};
use crate::j::J;

#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    J(J),
    Fn {
        stmts: Vec<AstNode>,
        arg_names: Vec<String>,
        fn_body: String,
        start: usize,
        source_id: usize,
    },
    UnaryOp {
        op: Box<AstNode>,
        exp: Box<AstNode>,
    },
    BinOp {
        op: Box<AstNode>,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    Assign {
        id: String,
        exp: Box<AstNode>,
    },
    IndexAssign {
        id: String,
        indices: Vec<AstNode>,
        exp: Box<AstNode>,
    },
    Op {
        op: String,
        start: usize,
        source_id: usize,
    },
    Id {
        id: String,
        start: usize,
        source_id: usize,
    },
    Call {
        f: Box<AstNode>,
        args: Vec<AstNode>,
        start: usize,
        source_id: usize,
    },
    If {
        cond: Box<AstNode>,
        stmts: Vec<AstNode>,
    },
    While {
        cond: Box<AstNode>,
        stmts: Vec<AstNode>,
    },
    Try {
        tries: Vec<AstNode>,
        catches: Vec<AstNode>,
    },
    Return(Box<AstNode>),
    Raise(Box<AstNode>),
    Dataframe(Vec<AstNode>),
    Matrix(Vec<AstNode>),
    Dict {
        keys: Vec<String>,
        values: Vec<AstNode>,
    },
    List(Vec<AstNode>),
    Series {
        name: String,
        exp: Box<AstNode>,
    },
    Sql {
        op: String,
        from: Box<AstNode>,
        filters: Vec<AstNode>,
        groups: Vec<AstNode>,
        ops: Vec<AstNode>,
        sorts: Vec<AstNode>,
        take: Box<AstNode>,
        source_id: usize,
        start: usize,
    },
    Skip,
}

impl AstNode {
    pub fn as_j(self) -> JResult<J> {
        if let AstNode::J(j) = self {
            Ok(j)
        } else {
            Err(JError::ParserErr("Failed to cast to J".to_owned()))
        }
    }
}
