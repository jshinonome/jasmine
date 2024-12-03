from .ast import AstSql, downcast_ast_node
from .context import Context
from .engine import Engine
from .exceptions import JasmineEvalException

SQL_FN = {
    # operators
    "!=": 2,
    "<=": 2,
    ">=": 2,
    ">": 2,
    "<": 2,
    "==": 2,
    "@": 2,
    # cast
    "$": 2,
    "?": 2,
    "++": 2,
    "+": 2,
    "-": 2,
    # pow
    "**": 2,
    "*": 2,
    # floor division
    "//": 2,
    "/": 2,
    # mod
    "%": 2,
    "|": 2,
    "&": 2,
    # take
    "#": 2,
    # xor
    "^": 2,
    # unary
    "abs": 1,
    "all": 1,
    "any": 1,
    # arc functions
    "acos": 1,
    "acosh": 1,
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
def eval_sql(sql: AstSql, engine: Engine, ctx: Context, source_id: int, start: int):
    try:
        pass
    except Exception as e:
        raise JasmineEvalException(engine.get_trace(source_id, start, str(e)))
