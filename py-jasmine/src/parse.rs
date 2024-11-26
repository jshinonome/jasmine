use jasmine::{parse, AstNode};
use pyo3::{pyclass, pyfunction, types::PyList, IntoPy, PyObject, PyResult, Python, ToPyObject};

use crate::{error::JasmineParseError, j::JObj};

#[pyclass]
pub struct AstSkip();

#[pyclass]
pub struct AstId {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    start: usize,
    #[pyo3(get)]
    source_id: usize,
}

#[pyclass]
pub struct AstFn {
    #[pyo3(get)]
    stmts: Vec<PyObject>,
    #[pyo3(get)]
    arg_names: Vec<String>,
    #[pyo3(get)]
    args: Vec<PyObject>,
    #[pyo3(get)]
    fn_body: String,
    #[pyo3(get)]
    start: usize,
    #[pyo3(get)]
    source_id: usize,
}

#[pyclass]
pub struct AstUnaryOp {
    #[pyo3(get)]
    op: PyObject,
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstBinOp {
    #[pyo3(get)]
    op: PyObject,
    #[pyo3(get)]
    lhs: PyObject,
    #[pyo3(get)]
    rhs: PyObject,
}

#[pyclass]
pub struct AstAssign {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstIndexAssign {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    indices: Vec<PyObject>,
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstOp {
    #[pyo3(get)]
    op: String,
    #[pyo3(get)]
    start: usize,
    #[pyo3(get)]
    source_id: usize,
}

#[pyclass]
pub struct AstCall {
    #[pyo3(get)]
    f: PyObject,
    #[pyo3(get)]
    args: Vec<PyObject>,
}

#[pyclass]
pub struct AstIf {
    #[pyo3(get)]
    cond: PyObject,
    #[pyo3(get)]
    stmts: Vec<PyObject>,
}

#[pyclass]
pub struct AstWhile {
    #[pyo3(get)]
    cond: PyObject,
    #[pyo3(get)]
    stmts: Vec<PyObject>,
}

#[pyclass]
pub struct AstTry {
    #[pyo3(get)]
    tries: Vec<PyObject>,
    #[pyo3(get)]
    catches: Vec<PyObject>,
}

#[pyclass]
pub struct AstReturn {
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstRaise {
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstDataFrame {
    #[pyo3(get)]
    exps: Vec<PyObject>,
}

#[pyclass]
pub struct AstMatrix {
    #[pyo3(get)]
    exps: Vec<PyObject>,
}

#[pyclass]
pub struct AstDict {
    #[pyo3(get)]
    keys: Vec<String>,
    #[pyo3(get)]
    values: Vec<PyObject>,
}

#[pyclass]
pub struct AstList {
    #[pyo3(get)]
    exps: Vec<PyObject>,
}

#[pyclass]
pub struct AstSeries {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    exp: PyObject,
}

#[pyclass]
pub struct AstSql {
    #[pyo3(get)]
    op: String,
    #[pyo3(get)]
    from: PyObject,
    #[pyo3(get)]
    filters: Vec<PyObject>,
    #[pyo3(get)]
    groups: Vec<PyObject>,
    #[pyo3(get)]
    ops: Vec<PyObject>,
    #[pyo3(get)]
    sorts: Vec<PyObject>,
    #[pyo3(get)]
    take: PyObject,
}

#[pyclass]
pub enum AstType {
    Dataframe,
    Matrix,
    Dict,
    List,
    Series,
    Sql,
}

#[pyfunction]
pub fn parse_source_code<'a>(
    py: Python<'a>,
    source_code: &str,
    source_id: usize,
) -> PyResult<PyObject> {
    let ast_nodes =
        parse(source_code, source_id).map_err(|e| JasmineParseError::new_err(e.to_string()))?;

    Ok(PyList::new_bound(py, ast_nodes.into_iter().map(|n| cast_ast_to_py(py, n))).into())
}

pub fn cast_ast_to_py<'a>(py: Python<'a>, node: AstNode) -> PyObject {
    match node {
        AstNode::J(j) => JObj::new(j).into_py(py),
        AstNode::Fn {
            stmts,
            arg_names,
            args,
            fn_body,
            start,
            source_id,
        } => {
            let stmts = stmts.into_iter().map(|n| cast_ast_to_py(py, n)).collect();
            let args = args.into_iter().map(|n| cast_ast_to_py(py, n)).collect();
            AstFn {
                stmts,
                arg_names,
                args,
                fn_body,
                start,
                source_id,
            }
            .into_py(py)
        }
        AstNode::UnaryOp { op, exp } => AstUnaryOp {
            op: cast_ast_to_py(py, *op),
            exp: cast_ast_to_py(py, *exp),
        }
        .into_py(py),
        AstNode::BinOp { op, lhs, rhs } => AstBinOp {
            op: cast_ast_to_py(py, *op),
            lhs: cast_ast_to_py(py, *lhs),
            rhs: cast_ast_to_py(py, *rhs),
        }
        .into_py(py),
        AstNode::Assign { id, exp } => AstAssign {
            id,
            exp: cast_ast_to_py(py, *exp),
        }
        .into_py(py),
        AstNode::IndexAssign { id, indices, exp } => AstIndexAssign {
            id,
            indices: indices.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            exp: cast_ast_to_py(py, *exp),
        }
        .into_py(py),
        AstNode::Op {
            op,
            start,
            source_id,
        } => AstOp {
            op,
            start,
            source_id,
        }
        .into_py(py),
        AstNode::Id {
            id,
            start,
            source_id,
        } => AstId {
            id,
            start,
            source_id,
        }
        .into_py(py),
        AstNode::Call { f, args } => AstCall {
            f: cast_ast_to_py(py, *f),
            args: args.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::If { cond, stmts } => AstIf {
            cond: cast_ast_to_py(py, *cond),
            stmts: stmts.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::While { cond, stmts } => AstWhile {
            cond: cast_ast_to_py(py, *cond),
            stmts: stmts.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::Try { tries, catches } => AstTry {
            tries: tries.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            catches: catches.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::Return(n) => AstReturn {
            exp: cast_ast_to_py(py, *n),
        }
        .into_py(py),
        AstNode::Raise(n) => AstRaise {
            exp: cast_ast_to_py(py, *n),
        }
        .into_py(py),
        AstNode::Dataframe(vec) => AstDataFrame {
            exps: vec.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::Matrix(vec) => AstMatrix {
            exps: vec.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::Dict { keys, values } => AstDict {
            keys: keys,
            values: values.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::List(vec) => AstList {
            exps: vec.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
        }
        .into_py(py),
        AstNode::Series { name, exp } => AstSeries {
            name,
            exp: cast_ast_to_py(py, *exp),
        }
        .into_py(py),
        AstNode::Sql {
            op,
            from,
            filters,
            groups,
            ops,
            sorts,
            take,
        } => AstSql {
            op,
            from: cast_ast_to_py(py, *from),
            filters: filters.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            groups: groups.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            ops: ops.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            sorts: sorts.into_iter().map(|n| cast_ast_to_py(py, n)).collect(),
            take: take
                .map(|n| cast_ast_to_py(py, *n))
                .unwrap_or(().to_object(py)),
        }
        .into_py(py),
        AstNode::Skip => AstSkip {}.into_py(py),
    }
}
