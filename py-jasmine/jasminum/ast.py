import contextlib
from enum import Enum

with contextlib.suppress(ImportError):
    from jasminum.jasminum import (
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
        AstUnaryOp,
        AstWhile,
        JObj,
        parse_source_code,
        print_trace,
    )


class AstType(Enum):
    J = 0
    Fn = 1
    UnaryOp = 2
    BinOp = 3
    Assign = 4
    IndexAssign = 5
    Op = 6
    Id = 7
    Call = 8
    If = 9
    While = 10
    Try = 11
    Return = 12
    Raise = 13
    Dataframe = 14
    Matrix = 15
    Dict = 16
    List = 17
    Series = 18
    Sql = 19
    Skip = 20


all = [
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
    AstUnaryOp,
    AstWhile,
    JObj,
    parse_source_code,
    print_trace,
]
