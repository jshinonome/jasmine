use crate::errors::{JError, JResult};
use crate::j::J;

#[derive(PartialEq, Debug, Clone)]
pub enum AstNode {
    J(J),
    Fn {
        f: Vec<AstNode>,
        arg_names: Vec<String>,
        args: Vec<AstNode>,
        start: usize,
        end: usize,
        source_id: usize,
    },
    UnaryExp {
        f: Box<AstNode>,
        exp: Box<AstNode>,
    },
    BinaryExp {
        f: Box<AstNode>,
        lhs: Box<AstNode>,
        rhs: Box<AstNode>,
    },
    AssignmentExp {
        id: String,
        exp: Box<AstNode>,
    },
    IndexAssignmentExp {
        id: String,
        indices: Vec<AstNode>,
        exp: Box<AstNode>,
    },
    Operator {
        op: String,
        start: usize,
        source_id: usize,
    },
    Id {
        id: String,
        start: usize,
        source_id: usize,
    },
    FnCall {
        f: Box<AstNode>,
        args: Vec<AstNode>,
    },
    If {
        cond: Box<AstNode>,
        nodes: Vec<AstNode>,
    },
    While {
        cond: Box<AstNode>,
        nodes: Vec<AstNode>,
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
    SeriesExp {
        name: String,
        exp: Box<AstNode>,
    },
    Sql {
        op: String,
        from_exp: Box<AstNode>,
        filter_exp: Vec<AstNode>,
        group_exp: Vec<AstNode>,
        op_exp: Vec<AstNode>,
        sort_exp: Vec<AstNode>,
        take_exp: Option<Box<AstNode>>,
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
