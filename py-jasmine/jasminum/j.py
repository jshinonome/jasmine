from enum import Enum

from .ast import JObj
from .j_fn import JFn


class JType(Enum):
    NONE = 0
    BOOLEAN = 1
    INT = 2
    DATE = 3
    TIME = 4
    DATETIME = 5
    TIMESTAMP = 6
    DURATION = 7
    FLOAT = 8
    STRING = 9
    SYMBOL = 10
    SERIES = 11
    MATRIX = 12
    LIST = 13
    DICT = 14
    DATAFRAME = 15
    ERR = 16
    FN = 17
    MISSING = 18
    RETURN = 19


class J:
    data: any
    j_type: JType

    def __init__(self, data, j_type=JType.NONE) -> None:
        self.data = data
        if isinstance(data, JObj):
            self.j_type = JType(data.j_type)
        elif isinstance(data, JFn):
            self.j_type = JType.FN
        else:
            self.j_type = j_type

    def __str__(self) -> str:
        match self.j_type:
            case JType.INT | JType.FLOAT:
                return f"{self.data}"
