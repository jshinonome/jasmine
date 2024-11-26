import contextlib

from jasminum.engine import Engine

with contextlib.suppress(ImportError):
    from jasminum.jasminum import (
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
        parse_source_code,
    )


def eval(source_code: str, engine: Engine):
    pass
