from datetime import timedelta

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
