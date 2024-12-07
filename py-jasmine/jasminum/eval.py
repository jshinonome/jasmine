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
from .constant import PL_DATA_TYPE
from .context import Context
from .engine import Engine
from .exceptions import JasmineEvalException
from .j import J, JType
from .j_fn import JFn
from .util import date_to_num


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
        if node.name in engine.builtins:
            return engine.builtins[node.name]
        elif node.name in ctx.locals:
            return ctx.locals[node.name]
        elif node.name in engine.globals:
            return engine.globals[node.name]
        else:
            raise JasmineEvalException(
                engine.get_trace(
                    node.source_id, node.start, "'%s' is not defined" % node.id
                )
            )
    elif isinstance(node, AstUnaryOp):
        op = downcast_ast_node(node.op)
        op_fn = eval_node(op, engine, ctx, is_in_fn)
        exp = eval_node(node.exp, engine, ctx, is_in_fn)
        return eval_fn(op_fn, engine, ctx, op.source_id, op.start, exp)
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
        if node.name in engine.builtins:
            return engine.builtins.get(node.name)
        elif node.name in engine.globals:
            return engine.globals.get(node.name)
        else:
            raise JasmineEvalException(
                engine.get_trace(
                    node.source_id, node.start, "'%s' is not defined" % node.name
                )
            )
    elif isinstance(node, AstFn):
        raise JasmineEvalException("not yet implemented")
    elif isinstance(node, AstSql):
        return eval_sql(node, engine, ctx, node.source_id, node.start, is_in_fn)
    else:
        raise JasmineEvalException("not yet implemented - %s" % node)


def eval_fn(fn: JFn, engine: Engine, ctx: Context, source_id: int, start: int, *args):
    try:
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
    except Exception as e:
        raise JasmineEvalException(engine.get_trace(source_id, start, str(e)))


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
    "++": lambda x, y: pl.concat_list([x, y]),
    "+": pl.Expr.add,
    "-": pl.Expr.sub,
    # pow
    "**": pl.Expr.pow,
    "*": pl.Expr.mul,
    # floor division
    "div": pl.Expr.floordiv,
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
    "asin": pl.Expr.arcsin,
    "asinh": pl.Expr.arcsinh,
    "atan": pl.Expr.arctan,
    "atanh": pl.Expr.arctanh,
    # sort asc
    "asc": pl.Expr.sort,
    # backward fill
    "bfill": lambda x: pl.Expr.backward_fill(x),
    "cbrt": pl.Expr.cbrt,
    "ceil": pl.Expr.ceil,
    "cos": pl.Expr.cos,
    "cosh": pl.Expr.cosh,
    "cot": pl.Expr.cot,
    "count": pl.Expr.count,
    # cumulative functions
    "ccount": pl.Expr.cum_count,
    "cmax": pl.Expr.cum_max,
    "cmin": pl.Expr.cum_min,
    "cprod": pl.Expr.cum_prod,
    "csum": pl.Expr.cum_sum,
    # sort desc
    "desc": lambda x: pl.Expr.sort(x, descending=True),
    "diff": lambda x: pl.Expr.diff(x),
    "exp": pl.Expr.exp,
    "first": pl.Expr.first,
    "flatten": pl.Expr.flatten,
    "floor": pl.Expr.floor,
    # forward fill
    "ffill": lambda x: pl.Expr.forward_fill(x),
    "hash": lambda x: pl.Expr.hash(x),
    # interpolate
    "interp": lambda x: pl.Expr.interpolate(x),
    "kurtosis": pl.Expr.kurtosis,
    "last": pl.Expr.last,
    "ln": lambda x: pl.Expr.log(x),
    "log10": pl.Expr.log10,
    "log1p": pl.Expr.log1p,
    "lowercase": lambda x: x.str.to_lowercase(),
    # strip start
    "strips": lambda x: x.str.strip_chars_start(),
    "max": pl.Expr.max,
    "mean": pl.Expr.mean,
    "median": pl.Expr.median,
    "min": pl.Expr.min,
    "neg": pl.Expr.neg,
    "next": lambda x: pl.Expr.shift(x, -1),
    "mode": pl.Expr.mode,
    "not": pl.Expr.not_,
    "null": pl.Expr.is_null,
    # percent change
    "pc": lambda x: pl.Expr.pct_change(x),
    "prev": lambda x: pl.Expr.shift(x),
    "prod": pl.Expr.product,
    "rank": lambda x: pl.Expr.rank(x),
    "reverse": pl.Expr.reverse,
    # strip end
    "stripe": lambda x: x.str.strip_chars_end(),
    "shuffle": pl.Expr.shuffle,
    "sign": pl.Expr.sign,
    "sin": pl.Expr.sin,
    "sinh": pl.Expr.sinh,
    "skew": pl.Expr.skew,
    "sqrt": pl.Expr.sqrt,
    "std0": lambda x: pl.Expr.std(x, 0),
    "std1": lambda x: pl.Expr.std(x, 1),
    "string": lambda x: x.cast(pl.String),
    "strip": lambda x: x.str.strip_chars(),
    "sum": pl.Expr.sum,
    "tan": pl.Expr.tan,
    "tanh": pl.Expr.tanh,
    "unique": pl.Expr.unique,
    # unique count
    "uc": pl.Expr.unique_counts,
    "uppercase": lambda x: x.str.to_uppercase(),
    "var0": lambda x: pl.Expr.var(x, 0),
    "var1": lambda x: pl.Expr.var(x, 1),
    # binary
    "between": 2,
    # bottom k
    "bottom": lambda x, y: pl.Expr.bottom_k(y, x),
    "corr0": lambda x, y: pl.corr(x, y, ddof=0),
    "corr1": lambda x, y: pl.corr(x, y, ddof=1),
    "cov0": lambda x, y: pl.cov(x, y, 0),
    "cov1": lambda x, y: pl.cov(x, y, 1),
    "differ": lambda x, y: x.list.set_difference(y),
    # ewm functions
    "emean": lambda x, y: pl.Expr.ewm_mean(y, alpha=x),
    "estd": lambda x, y: pl.Expr.ewm_std(y, alpha=x),
    "evar": lambda x, y: pl.Expr.ewm_mean(y, alpha=x),
    # fill null
    "fill": 2,
    "in": pl.Expr.is_in,
    "intersect": 2,
    "like": 2,
    "log": lambda x, y: pl.Expr.log(x, y),
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
    "over": pl.Expr.over,
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
            if len(sql.filters) > 0:
                for node in sql.filters:
                    df = df.filter(eval_sql_op(node, engine, ctx, is_in_fn))
        elif j.j_type == JType.PARTED:
            missing_part_err = JasmineEvalException(
                "dataframe partitioned by %s requires its partitioned unit condition('==', 'in' or 'between') as its first filter clause"
                % j.j_type.name
            )
            # partitioned table
            if len(sql.filters) > 0:
                first_filter = downcast_ast_node(sql.filters[0])
                if isinstance(first_filter, AstBinOp):
                    op = downcast_ast_node(first_filter.op)
                    lhs = downcast_ast_node(first_filter.lhs)
                    rhs = eval_node(first_filter.rhs, engine, ctx, is_in_fn)
                    if not (isinstance(lhs, AstId) and lhs.name == j.data.get_unit()):
                        raise missing_part_err
                    if op.name == "==" and rhs.j_type == JType.DATE:
                        date_num = rhs.date_num()
                        partitions = j.data.get_partition_paths(date_num, date_num)
                        if len(partitions) == 0:
                            partitions = [j.data.get_latest_path()]
                            df = pl.scan_parquet(partitions, n_rows=0)
                        else:
                            df = pl.scan_parquet(partitions)
                    elif (
                        op.name == "between"
                        and rhs.j_type == JType.SERIES
                        and rhs.data.dtype == pl.Date
                        and rhs.data.count() == 2
                        and rhs.data.null_count() == 0
                    ):
                        start_date = rhs.data[0]
                        end_date = rhs.data[1]
                        partitions = j.data.get_partition_paths(
                            date_to_num(start_date), date_to_num(end_date)
                        )
                        if len(partitions) == 0:
                            partitions = [j.data.get_latest_path()]
                            df = pl.scan_parquet(partitions, n_rows=0)
                        else:
                            df = pl.scan_parquet(partitions)
                    elif op.name == "in" and rhs.j_type == JType.DATE:
                        date_num = date_to_num(rhs.data)
                        if date_num in j.data.partitions:
                            df = pl.scan_parquet(
                                j.data.get_partition_paths_by_date_nums([date_num])
                            )
                        else:
                            df = pl.scan_parquet([j.data.get_latest_path()], n_rows=0)
                    elif op.name == "in" and (
                        rhs.j_type == JType.SERIES and rhs.data.dtype == pl.Date
                    ):
                        date_nums = []
                        for date in rhs.data:
                            if date:
                                date_num = date_to_num(date)
                                if date_num in j.data.partitions:
                                    date_nums.append(date_num)
                        partitions = j.data.get_partition_paths_by_date_nums(date_nums)
                        if len(partitions) > 0:
                            df = pl.scan_parquet(partitions)
                        else:
                            df = pl.scan_parquet([j.data.get_latest_path()], n_rows=0)
                    else:
                        raise missing_part_err
                else:
                    raise missing_part_err
            else:
                raise missing_part_err

            if len(sql.filters) > 1:
                for node in sql.filters[1:]:
                    df = df.filter(eval_sql_op(node, engine, ctx, is_in_fn))
        else:
            raise JasmineEvalException("'from' requires dataframe, got %s" % j.j_type)

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

        sorts = []
        descendings = []
        if len(sql.sorts) > 0:
            for sort in sql.sorts:
                sort = downcast_ast_node(sort).id
                if sort.startswith("-"):
                    sorts.append(sort[1:])
                    descendings.append(True)
                else:
                    sorts.append(sort)
                    descendings.append(False)
            df = df.sort(sorts, descending=descendings)

        take = eval_sql_op(sql.take, engine, ctx, is_in_fn)

        if (
            isinstance(take, J)
            and take.j_type == JType.INT
            or take.j_type == JType.NONE
        ):
            if take.j_type == JType.INT:
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
        if node.name == "i":
            return pl.int_range(pl.len(), dtype=pl.UInt32).alias("i")
        elif node.name in engine.builtins:
            return engine.builtins[node.name]
        elif node.name in ctx.locals:
            return ctx.locals[node.name]
        elif node.name in engine.globals:
            return engine.globals[node.name]
        else:
            return pl.col(node.name)
    elif isinstance(node, AstUnaryOp):
        op = downcast_ast_node(node.op)
        exp = eval_sql_op(node.exp, engine, ctx, is_in_fn)
        if isinstance(exp, J):
            op_fn = eval_node(op, engine, ctx, is_in_fn)
            return eval_fn(
                op_fn,
                engine,
                ctx,
                op.source_id,
                op.start,
                exp,
            )
        else:
            # AstOp | AstId
            fn_name = op.name
            op_fn = get_sql_fn(op, fn_name, 1, engine)
            return eval_sql_fn(op_fn, fn_name, exp)
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
            op_fn = get_sql_fn(op, op.name, 2, engine)
            return eval_sql_fn(op_fn, op.name, lhs, rhs)
    else:
        raise JasmineEvalException("not yet implemented for sql - %s" % node)


def eval_sql_cast(type_name: str, expr: pl.Expr):
    match type_name:
        case type_name if type_name in PL_DATA_TYPE:
            return expr.cast(PL_DATA_TYPE(type_name))
        case "year":
            return expr.dt.year()
        case "month":
            return expr.dt.month()
        case "month_start":
            return expr.dt.month_start()
        case "month_end":
            return expr.dt.month_end()
        case "weekday":
            return expr.dt.weekday()
        case "day":
            return expr.dt.day()
        case "dt":
            return expr.dt.date()
        case "hour":
            return expr.dt.hour()
        case "minute":
            return expr.dt.minute()
        case "second":
            return expr.dt.second()
        case "t":
            return expr.dt.time()
        case "ms":
            return expr.dt.millisecond()
        case "ns":
            return expr.dt.nanosecond()
        case _:
            raise JasmineEvalException("unknown data type %s" % type_name)


def eval_sql_fn(fn: Callable, fn_name: str, *args) -> pl.Expr:
    match fn_name:
        case "$":
            j = args[0]
            expr = args[1]
            if (
                isinstance(j, J)
                and (j.j_type == JType.STRING or j.j_type == JType.CAT)
                and isinstance(expr, pl.Expr)
            ):
                datatype = j.data
                return eval_sql_cast(datatype, expr)
            else:
                raise JasmineEvalException(
                    "'$'(cast) requires data type and series expression"
                )
        case _:
            fn_args = []
            for arg in args:
                if isinstance(arg, J):
                    fn_args.append(arg.to_expr())
                else:
                    fn_args.append(arg)
            return fn(*fn_args)


def get_sql_fn(node: AstOp | AstId, fn_name: str, arg_num: int, engine: Engine):
    if fn_name in SQL_FN:
        fn = SQL_FN[fn_name]
        if fn.__code__.co_argcount != arg_num:
            raise JasmineEvalException(
                engine.get_trace(
                    node.source_id,
                    node.start,
                    "'%s' takes %s arguments but %s were given"
                    % (fn_name, fn.__code__.co_argcount, arg_num),
                )
            )

        return fn
    else:
        raise JasmineEvalException(
            engine.get_trace(
                node.source_id,
                node.start,
                "%s is not a valid sql fn" % fn_name,
            )
        )
