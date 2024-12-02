from datetime import date, datetime, timezone
from enum import Enum
from typing import Callable

import polars as pl

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
    data: JObj | date | int | float | pl.Series | pl.DataFrame
    j_type: JType

    def __init__(self, data, j_type=JType.NONE) -> None:
        self.data = data
        if isinstance(data, JObj):
            self.j_type = JType(data.j_type)
            match self.j_type:
                case JType.DATETIME | JType.TIMESTAMP:
                    self.data = data
                case _:
                    self.data = data.as_py()
        elif isinstance(data, pl.Series):
            self.j_type = JType.SERIES
        elif isinstance(data, JFn):
            self.j_type = JType.FN
        elif isinstance(data, date):
            self.j_type = JType.DATE
        else:
            self.j_type = j_type

    def __str__(self) -> str:
        match JType(self.j_type):
            case JType.INT | JType.FLOAT | JType.SERIES | JType.DATAFRAME:
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
                return self.data.format_temporal()
            case JType.TIMESTAMP:
                return self.data.format_temporal()
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

    def days(self) -> int:
        if self.j_type == JType.DURATION:
            return self.data // 86_400_000_000_000
        else:
            raise JasmineEvalException(
                "requires 'duration' for 'days', got %s" % repr(self.j_type)
            )

    def days_from_epoch(self) -> int:
        if self.j_type == JType.DATE:
            return self.data.toordinal() - 719_163
        else:
            raise JasmineEvalException(
                "requires 'date' for 'days from epoch', got %s" % repr(self.j_type)
            )

    def nanos_from_epoch(self) -> int:
        if self.j_type == JType.DATE:
            return (self.data.toordinal() - 719_163) * 86_400_000_000_000
        if self.j_type == JType.TIMESTAMP:
            return self.data.as_py()
        else:
            raise JasmineEvalException(
                "requires 'date' or 'timestamp' for 'nanos from epoch', got %s"
                % repr(self.j_type)
            )

    def __eq__(self, value: object) -> bool:
        if isinstance(value, J):
            if self.j_type != value.j_type:
                return False
            match self.j_type:
                case JType.DATETIME | JType.TIMESTAMP:
                    return (
                        self.data.tz() == self.data.tz()
                        and self.data.as_py() == self.data.as_py()
                    )
                case _:
                    return self.data == value.data
        else:
            return False

    def with_timezone(self, timezone: str):
        return J(self.data.with_timezone(timezone), self.j_type)

    @classmethod
    def from_nanos(cls, ns: int, timezone: str):
        return J(JObj(ns, timezone, "ns"))

    @classmethod
    def from_millis(cls, ms: int, timezone: str):
        return J(JObj(ms, timezone, "ms"))

    def tz(self) -> str:
        return self.data.tz()
