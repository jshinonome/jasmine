from datetime import date, datetime, timezone
from enum import Enum

from .ast import JObj
from .exceptions import JasmineEvalException
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
            self.data = data.as_py()
            self.j_type = JType(data.j_type)
        elif isinstance(data, JFn):
            self.j_type = JType.FN
        elif isinstance(data, date):
            self.j_type = JType.DATE
        else:
            self.j_type = j_type

    def __str__(self) -> str:
        match JType(self.j_type):
            case JType.INT | JType.FLOAT:
                return f"{self.data}"
            case JType.DATE:
                return self.data.isoformat()
            case JType.TIME:
                sss = self.data % 1000000000
                ss = self.data // 1000000000
                HH = ss // 3600
                mm = ss % 3600 // 60
                ss = ss % 60
                return f"{HH:02d}:{mm:02d}:{ss:02d}:{sss:09d}"
            case JType.DATETIME:
                return datetime.fromtimestamp(
                    self.data / 1000, timezone.utc
                ).isoformat()[:-9]
            case JType.TIMESTAMP:
                ns = self.data % 1000000000
                t = datetime.fromtimestamp(
                    self.data // 1000000000, timezone.utc
                ).isoformat()[:-6]
                t = t.replace("T", "D")
                return f"{t}.{ns:09d}"
            case JType.DURATION:
                neg = "" if self.data >= 0 else "-"
                ns = abs(self.data)
                sss = ns % 1000000000
                ss = ns // 1000000000
                mm = ss // 60
                ss = ss % 60
                HH = mm // 60
                mm = mm % 60
                days = HH // 24
                HH = HH % 24
                return f"{neg}{days}D{HH:02d}:{mm:02d}:{ss:02d}:{sss:09d}"
            case _:
                return repr(self)

    def __repr__(self) -> str:
        return "<%s - %s>" % (self.j_type.name, self.data)

    def int(self) -> int:
        return int(self.data)

    def days_from_epoch(self) -> int:
        if self.j_type == JType.DATE:
            return self.data.toordinal() - 719_163
        else:
            raise JasmineEvalException(
                "Failed to refer 'days' from %s" % repr(self.j_type)
            )

    def nanos_from_epoch(self) -> int:
        if self.j_type == JType.DATE:
            return (self.data.toordinal() - 719_163) * 86_400_000_000_000
        if self.j_type == JType.TIMESTAMP:
            return self.data
        else:
            raise JasmineEvalException(
                "Failed to refer 'nanos' from %s" % repr(self.j_type)
            )

    def __eq__(self, value: object) -> bool:
        if isinstance(value, J):
            return self.data == value.data and self.j_type == value.j_type
        else:
            return False
