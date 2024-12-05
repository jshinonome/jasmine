from datetime import timedelta

import numpy as np
import polars as pl

from .constant import PL_DATA_TYPE
from .exceptions import JasmineEvalException
from .j import J, JType


# |           | date | time | datetime | timestamp | duration  |
# | --------- | ---- | ---- | -------- | --------- | --------- |
# | date      | -    | -    | -        | -         | date      |
# | time      | -    | -    | -        | -         | -         |
# | datetime  | -    | -    | -        | -         | duration  |
# | timestamp | -    | -    | -        | -         | timestamp |
# | duration  | date | -    | datetime | timestamp | duration  |
def add(arg1: J, arg2: J) -> J:
    if isinstance(arg1, J) and isinstance(arg2, J):
        if arg1.j_type == JType.NONE or arg2.j_type == JType.NONE:
            return J(None, JType.NONE)
        elif arg1.j_type.value <= 2 and arg2.j_type.value <= 2:
            return J(arg1.data + arg2.data, JType.INT)
        elif arg1.j_type == JType.DATE and arg2.j_type == JType.DURATION:
            return J(arg1.data + timedelta(days=arg2.days()))
        elif arg1.j_type == JType.TIMESTAMP and arg2.j_type == JType.DURATION:
            return J.from_nanos(arg1.nanos_from_epoch() + arg2.data, arg1.tz())
        elif arg1.j_type == JType.DATETIME and arg2.j_type == JType.DURATION:
            return J.from_millis(arg1.data + arg2.data // 1000000, arg1.tz())
        elif arg1.j_type == JType.DURATION and arg2.j_type == JType.DURATION:
            return J(arg1.data + arg2.data, JType.DURATION)
        elif (
            (arg1.j_type.value <= 2 and arg2.j_type <= 7)
            or (
                arg2.j_type in (JType.DATE, JType.TIMESTAMP)
                and arg1.j_type in (JType.TIME, JType.DURATION)
            )
            or (
                arg2.j_type == JType.DATETIME
                and (arg1.j_type in (JType.TIME, JType.DURATION))
            )
        ):
            return add(arg2, arg1)
        else:
            raise JasmineEvalException(
                "unsupported operand type(s) for '{0}': '{1}' and '{2}'".format(
                    "add", arg1.j_type.name, arg2.j_type.name
                )
            )


def rand(size: J, limit: J) -> J:
    if limit.j_type == JType.INT and size.j_type == JType.INT:
        return J(pl.Series("", np.random.randint(limit.data, size=size.data)))
    elif limit.j_type == JType.FLOAT and size.j_type == JType.INT:
        return J(pl.Series("", limit.data * np.random.rand(size.data)))
    else:
        raise JasmineEvalException(
            "'rand' requires 'int' and 'int|float', got '%s' and '%s'"
            % (size.j_type, size.j_type)
        )


def cast(type_name: J, arg: J) -> J:
    if type_name.j_type == JType.CAT or type_name.j_type == JType.STRING:
        name = type_name.data
        if name not in PL_DATA_TYPE and name not in [
            "year",
            "month",
            "month_start",
            "month_end",
            "weekday",
            "day",
            "dt",
            "hour",
            "minute",
            "second",
            "ms",
            "ns",
        ]:
            raise JasmineEvalException("unknown data type for 'cast': %s" % name)
        if arg.j_type == JType.SERIES:
            if name in PL_DATA_TYPE:
                return J(arg.data.cast(PL_DATA_TYPE[name]))
            else:
                match name:
                    case "year":
                        return J(arg.data.dt.year())
                    case "month":
                        return J(arg.data.dt.month())
                    case "month_start":
                        return J(arg.data.dt.month_start())
                    case "month_end":
                        return J(arg.data.dt.month_end())
                    case "weekday":
                        return J(arg.data.dt.weekday())
                    case "day":
                        return J(arg.data.dt.day())
                    case "dt":
                        return J(arg.data.dt.date())
                    case "hour":
                        return J(arg.data.dt.hour())
                    case "minute":
                        return J(arg.data.dt.minute())
                    case "second":
                        return J(arg.data.dt.second())
                    case "t":
                        return J(arg.data.dt.time())
                    case "ms":
                        return J(arg.data.dt.millisecond())
                    case "ns":
                        return J(arg.data.dt.nanosecond())
    else:
        raise JasmineEvalException(
            "'cast' requires 'cat|string' as data type, got %s" % type_name.j_type.name
        )
