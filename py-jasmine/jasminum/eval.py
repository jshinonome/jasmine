from copy import copy
from typing import Callable

import polars as pl

from .ast import (
    Ast,
    AstAssign,
    AstBinOp,
    AstCall,
    AstDataFrame,
    AstDict,
    AstFn,
    AstId,
    AstIf,
    AstIndexAssign,
    AstList,
    AstMatrix,
    AstOp,
    AstRaise,
    AstReturn,
    AstSeries,
    AstSkip,
    AstSql,
    AstTry,
    AstType,
    AstUnaryOp,
    AstWhile,
    JObj,
    downcast_ast_node,
    parse_source_code,
)
from .context import Context
from .engine import Engine
from .exceptions import JasmineEvalException
from .j import J, JType
from .j_fn import JFn


def import_path(path: str, engine: Engine):
    pass


def eval_src(source_code: str, source_id: int, engine: Engine, ctx: Context) -> J:
    nodes = parse_source_code(source_code, source_id)
    res = J(None, JType.NONE)
    for node in nodes:
        res = eval_node(node, engine, ctx, False)
        if res == JType.RETURN:
            return res
    return res


def eval_node(node, engine: Engine, ctx: Context, is_in_fn=False) -> J:
    if isinstance(node, Ast):
        node = downcast_ast_node(node)

    if isinstance(node, JObj):
        return J(node, node.j_type)
    elif isinstance(node, AstAssign):
        res = eval_node(node.exp, engine, ctx, is_in_fn)
        if is_in_fn and "." not in node.id:
            ctx.locals[node.id] = res
        else:
            engine.globals[node.id] = res
        return res
    elif isinstance(node, AstId):
        if node.id in engine.builtins:
            return engine.builtins[node.id]
        elif node.id in ctx.locals:
            return ctx.locals[node.id]
        elif node.id in engine.globals:
            return engine.globals[node.id]
        else:
            raise JasmineEvalException(
                engine.get_trace(
                    node.source_id, node.start, "'%s' is not defined" % node.id
                )
            )
    elif isinstance(node, AstBinOp):
        op = downcast_ast_node(node.op)
        op_fn = eval_node(op, engine, ctx, is_in_fn)
        lhs = eval_node(node.lhs, engine, ctx, is_in_fn)
        rhs = eval_node(node.rhs, engine, ctx, is_in_fn)
        return eval_fn(
            op_fn,
            engine,
            ctx,
            op.source_id,
            op.start,
            lhs,
            rhs,
        )
    elif isinstance(node, AstOp):
        if node.op in engine.builtins:
            return engine.builtins.get(node.op)
        elif node.op in engine.globals:
            return engine.globals.get(node.op)
        else:
            raise JasmineEvalException(
                engine.get_trace(
                    node.source_id, node.start, "'%s' is not defined" % node.op
                )
            )
    elif isinstance(node, AstFn):
        raise JasmineEvalException("not yet implemented")
    elif isinstance(node, AstSql):
        return eval_sql(node, engine, ctx, node.source_id, node.start, is_in_fn)
    else:
        raise JasmineEvalException("not yet implemented - %s" % node)


def eval_fn(fn: JFn, engine: Engine, ctx: Context, source_id: int, start: int, *args):
    if fn.arg_num < len(args):
        raise engine.get_trace(
            source_id,
            start,
            "takes %s arguments but %s were given" % (fn.arg_num, len(args)),
        )

    fn_args = fn.args
    missing_arg_names = fn.arg_names.copy()
    missing_arg_num = 0
    for i, arg in enumerate(args):
        if arg.j_type == JType.MISSING:
            missing_arg_num += 1
        else:
            fn_args[fn.arg_names[i]] = arg
            missing_arg_names.remove(fn.arg_names[i])

    if missing_arg_num == 0 and fn.arg_num == len(args):
        if isinstance(fn.fn, Callable):
            return fn.fn(**fn_args)
        else:
            return eval_node(fn.fn, engine, Context(fn_args), True)
    else:
        new_fn = copy(fn)
        new_fn.arg_names = missing_arg_names
        new_fn.arg_num = len(missing_arg_names)
        new_fn.args = fn_args
        return new_fn


SQL_FN = {
    # operators
    "!=": pl.Expr.ne_missing,
    "<=": pl.Expr.le,
    ">=": pl.Expr.ge,
    ">": pl.Expr.gt,
    "<": pl.Expr.lt,
    "==": pl.Expr.eq,
    "@": pl.Expr.get,
    # cast
    "$": pl.Expr.cast,
    "?": None,
    "++": lambda x, y: pl.concat_list([x, y]),
    "+": pl.Expr.add,
    "-": pl.Expr.sub,
    # pow
    "**": pl.Expr.pow,
    "*": pl.Expr.mul,
    # floor division
    "//": pl.Expr.floordiv,
    "/": pl.Expr.truediv,
    # mod
    "%": pl.Expr.mod,
    "|": pl.Expr.max,
    "&": pl.Expr.min,
    # take
    "#": lambda x, y: pl.Expr.head(y, x) if x > 0 else pl.Expr.tail(y, x),
    # xor
    "^": pl.Expr.xor,
    # unary
    "abs": pl.Expr.abs,
    "all": pl.Expr.all,
    "any": pl.Expr.any,
    # arc functions
    "acos": pl.Expr.arccos,
    "acosh": pl.Expr.arccosh,
    "asin": 1,
    "asinh": 1,
    "atan": 1,
    "atanh": 1,
    # sort asc
    "asc": 1,
    # backward fill
    "bfill": 1,
    "cbrt": 1,
    "ceil": 1,
    "cos": 1,
    "cosh": 1,
    "cot": 1,
    "count": 1,
    # cumulative functions
    "ccount": 1,
    "cmax": 1,
    "cmin": 1,
    "cproduct": 1,
    "csum": 1,
    # sort desc
    "desc": 1,
    "diff": 1,
    "exp": 1,
    "first": 1,
    "flatten": 1,
    "floor": 1,
    # forward fill
    "ffill": 1,
    "hash": 1,
    # interpolate
    "interp": 1,
    "kurtosis": 1,
    "last": 1,
    "ln": 1,
    "log10": 1,
    "log1p": 1,
    "lowercase": 1,
    # strip start
    "strips": 1,
    "max": 1,
    "mean": 1,
    "median": 1,
    "min": 1,
    "neg": 1,
    "next": 1,
    "mode": 1,
    "not": 1,
    "null": 1,
    # percent change
    "pc": 1,
    "prev": 1,
    "product": 1,
    "rank": 1,
    "reverse": 1,
    # strip end
    "stripe": 1,
    "shuffle": 1,
    "sign": 1,
    "sin": 1,
    "sinh": 1,
    "skew": 1,
    "sqrt": 1,
    "std0": 1,
    "std1": 1,
    "string": 1,
    "strip": 1,
    "sum": 1,
    "tan": 1,
    "tanh": 1,
    "unique": 1,
    # unique count
    "uc": 1,
    "uppercase": 1,
    "var0": 1,
    "var1": 1,
    # binary
    "between": 2,
    # bottom k
    "bottom": 2,
    "corr0": 2,
    "corr1": 2,
    "cov0": 2,
    "cov1": 2,
    "cross": 2,
    "differ": 2,
    # ewm functions
    "emean": 2,
    "estd": 2,
    "evar": 2,
    # fill null
    "fill": 2,
    "in": 2,
    "intersect": 2,
    "like": 2,
    "log": 2,
    "matches": 2,
    "join": 2,
    # rolling functions
    "rmax": 2,
    "rmean": 2,
    "rmedian": 2,
    "rmin": 2,
    "rskew": 2,
    "rstd0": 2,
    "rstd1": 2,
    "rsum": 2,
    "rvar0": 2,
    "rvar1": 2,
    "quantile": 2,
    "rotate": 2,
    "round": 2,
    "shift": 2,
    "split": 2,
    # search sorted left
    "ss": 2,
    # search sorted right
    "ssr": 2,
    # top k
    "top": 2,
    "union": 2,
    "wmean": 2,
    "wsum": 2,
    # other functions
    "clip": 3,
    "concat": 3,
    "replace": 3,
    "rolling": 3,
    "rquantile": 3,
}


# op: String,
# from: Ast,
# filters: Vec<Ast>,
# groups: Vec<Ast>,
# ops: Vec<Ast>,
# sorts: Vec<Ast>,
# take: Ast,
def eval_sql(
    sql: AstSql,
    engine: Engine,
    ctx: Context,
    source_id: int,
    start: int,
    is_in_fn: bool,
):
    try:
        j = eval_node(sql.from_df, engine, ctx, is_in_fn)
        if j.j_type == JType.DATAFRAME:
            df = j.data.lazy()
        else:
            # partitioned table
            pass
        if len(sql.filters) > 0:
            for node in sql.filters:
                df = df.filter(eval_sql_op(node, engine, ctx, is_in_fn))

        groups = []
        if len(sql.groups) > 0:
            for node in sql.groups:
                groups.append(eval_sql_op(node, engine, ctx, is_in_fn))

        ops = []
        if len(sql.ops) > 0:
            for node in sql.ops:
                ops.append(eval_sql_op(node, engine, ctx, is_in_fn))

        if len(groups) > 0:
            if sql.op == "select":
                if len(ops) == 0:
                    df = df.group_by(groups, maintain_order=True).agg(
                        pl.col("*").last()
                    )
                else:
                    df = df.group_by(groups, maintain_order=True).agg(ops)
            elif sql.op == "update":
                over_ops = []
                for op in ops:
                    over_ops.append(op.over(groups))
                df.with_columns(over_ops)
            else:
                raise JasmineEvalException(
                    engine.get_trace(
                        source_id, start, "not support 'delete' with 'group'"
                    )
                )
        elif len(ops) > 0:
            if sql.op == "select":
                df = df.select(ops)
            elif sql.op == "update":
                df = df.with_columns(ops)
            else:
                df.drop(ops)

        take = eval_sql_op(sql.take, engine, ctx, is_in_fn)

        if isinstance(take, J) and take.j_type == JType.INT:
            df = df.head(take.data)
        else:
            raise JasmineEvalException(
                engine.get_trace(
                    source_id, start, "requires 'int' for 'take', got %s" % take
                )
            )
        return J(df.collect())
    except Exception as e:
        raise JasmineEvalException(engine.get_trace(source_id, start, str(e)))


def eval_sql_op(node, engine: Engine, ctx: Context, is_in_fn: bool) -> J | pl.Expr:
    if isinstance(node, Ast):
        node = downcast_ast_node(node)

    if isinstance(node, JObj):
        return J(node, node.j_type)
    elif isinstance(node, AstSeries):
        expr = eval_sql_op(node.exp, engine, ctx, is_in_fn)
        if isinstance(expr, J):
            expr = expr.to_expr()
        return expr.alias(node.name)
    elif isinstance(node, AstId):
        if node.id in engine.builtins:
            return engine.builtins[node.id]
        elif node.id in ctx.locals:
            return ctx.locals[node.id]
        elif node.id in engine.globals:
            return engine.globals[node.id]
        else:
            return pl.col(node.id)
    elif isinstance(node, AstBinOp):
        op = downcast_ast_node(node.op)
        lhs = eval_sql_op(node.lhs, engine, ctx, is_in_fn)
        rhs = eval_sql_op(node.rhs, engine, ctx, is_in_fn)
        if isinstance(lhs, J) and isinstance(rhs, J):
            op_fn = eval_node(op, engine, ctx, is_in_fn)
            return eval_fn(
                op_fn,
                engine,
                ctx,
                op.source_id,
                op.start,
                lhs,
                rhs,
            )
        else:
            op_fn = get_sql_fn(op, engine)
            return eval_sql_fn(op_fn, lhs, rhs)
    else:
        raise JasmineEvalException("not yet implemented for sql - %s" % node)


def eval_sql_fn(fn: Callable, *args) -> pl.Expr:
    fn_args = []
    for arg in args:
        if isinstance(arg, J):
            fn_args.append(arg.to_expr())
        else:
            fn_args.append(arg)
    return fn(*fn_args)


def get_sql_fn(op: AstOp, engine):
    if op.op in SQL_FN:
        return SQL_FN[op.op]
    else:
        engine.get_trace(
            op.source_id,
            op.start,
            "%s is not a valid sql fn" % op.op,
        )
