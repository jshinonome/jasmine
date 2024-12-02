from copy import copy
from typing import Callable

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


def eval_src(source_code: str, source_id: int, engine: Engine, ctx: Context) -> J:
    nodes = parse_source_code(source_code, source_id)
    res = J(None, JType.NONE)
    for node in nodes:
        res = eval_node(node, engine, ctx, False)
        if res == JType.RETURN:
            return res
    return res


def downcast_ast_node(node: Ast):
    ast_type = AstType(node.get_ast_type())
    match ast_type:
        case AstType.J:
            return node.j()
        case AstType.Fn:
            return node.fn()
        case AstType.UnaryOp:
            return node.unary_op()
        case AstType.BinOp:
            return node.bin_op()
        case AstType.Assign:
            return node.assign()
        case AstType.IndexAssign:
            return node.index_assign()
        case AstType.Op:
            return node.op()
        case AstType.Id:
            return node.id()
        case AstType.Call:
            return node.call()
        case AstType.If:
            return node.if_exp()
        case AstType.While:
            return node.while_exp()
        case AstType.Try:
            return node.try_exp()
        case AstType.Return:
            return node.return_exp()
        case AstType.Raise:
            return node.raise_exp()
        case AstType.Dataframe:
            return node.dataframe()
        case AstType.Matrix:
            return node.matrix()
        case AstType.Dict:
            return node.dict()
        case AstType.List:
            return node.list()
        case AstType.Series:
            return node.series()
        case AstType.Sql:
            return node.sql()
        case AstType.Skip:
            return node.skip()


def eval_node(node, engine: Engine, ctx: Context, is_in_fn=False):
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
    elif isinstance(node, AstId):
        if node.id in engine.builtins:
            return engine.builtins[node.id]
        elif node.id in ctx.locals:
            return ctx.locals[node.id]
        elif node.id in engine.globals:
            return engine.globals[node.id]
        else:
            raise JasmineEvalException(
                get_trace(
                    engine, node.source_id, node.start, "'%s' is not defined" % node.id
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
                get_trace(
                    engine, node.source_id, node.start, "'%s' is not defined" % node.op
                )
            )
    elif isinstance(node, AstFn):
        raise JasmineEvalException("not yet implemented")
    else:
        raise JasmineEvalException("not yet implemented - %s" % node)


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
            return fn.fn(**fn_args)
        else:
            return eval_node(fn.fn, engine, Context(fn_args), True)
    else:
        new_fn = copy(fn)
        new_fn.arg_names = missing_arg_names
        new_fn.arg_num = len(missing_arg_names)
        new_fn.args = fn_args
        return new_fn
