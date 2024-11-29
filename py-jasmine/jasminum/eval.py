from copy import copy
from typing import Callable

from .ast import (
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
    AstUnaryOp,
    AstWhile,
    JObj,
    parse_source_code,
    print_trace,
)
from .context import Context
from .engine import Engine
from .exceptions import JasmineEvalException
from .j import J, JType
from .j_fn import JFn


def get_trace(engine: Engine, source_id: int, pos: int, msg: str) -> str:
    source, path = engine.sources.get(source_id)
    return print_trace(source, path, pos, msg)


def import_path(path: str, engine: Engine):
    pass


def eval_src(source_code: str, source_id: int, engine: Engine, ctx: Context):
    nodes = parse_source_code(source_code, source_id)
    res = J(None, JType.NONE)
    for node in nodes:
        res = eval_node(node, engine, ctx, False)
        if res == JType.RETURN:
            return res
    return res


def eval_node(node, engine: Engine, ctx: Context, is_in_fn=False):
    if isinstance(node, AstAssign):
        res = eval_node(AstAssign.exp, engine, ctx, is_in_fn)
        if is_in_fn and "." not in AstAssign.id:
            ctx.locals[AstAssign.id] = res
        else:
            engine.globals[AstAssign.id] = res
    elif isinstance(node, AstBinOp):
        op = eval_node(AstBinOp.op, engine, ctx, is_in_fn)
        lhs = eval_node(AstBinOp.lhs, engine, ctx, is_in_fn)
        rhs = eval_node(AstBinOp.rhs, engine, ctx, is_in_fn)
        return eval_fn(
            op,
            engine,
            ctx,
            node.op.source_id,
            node.op.start,
            lhs,
            rhs,
        )
    elif isinstance(node, AstOp):
        if AstOp.op in engine.builtins:
            return engine.builtins.get(node.op)
        elif AstOp.op in engine.globals:
            return engine.globals.get(node.op)
        else:
            raise JasmineEvalException(
                get_trace(node.source_id, node.start, "'%s' is not defined" % node.op)
            )
    elif isinstance(node, AstFn):
        raise JasmineEvalException("not yet implemented")


def eval_fn(fn: JFn, engine: Engine, ctx: Context, source_id: int, start: int, *args):
    if fn.arg_num < len(args):
        raise get_trace(
            engine,
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
            return fn.fn(**{fn_args})
        else:
            return eval_node(fn.fn, engine, Context(fn_args), True)
    else:
        new_fn = copy(fn)
        new_fn.arg_names = missing_arg_names
        new_fn.arg_num = len(missing_arg_names)
        new_fn.args = fn_args
        return new_fn
