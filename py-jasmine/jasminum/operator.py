from datetime import timedelta

from .exceptions import JasmineEvalException
from .j import J, JType


def add(arg1: J, arg2: J) -> J:
    if isinstance(arg1, J) and isinstance(arg2, J):
        if arg1.j_type == JType.NONE or arg2.j_type == JType.NONE:
            return J(None, JType.NONE)
        elif arg1.j_type.value <= 2 and arg2.j_type.value <= 2:
            return J(arg1.data + arg2.data, JType.INT)
        elif arg1.j_type == JType.DATE and arg2.j_type.value <= 2:
            return J(arg1.data + timedelta(days=arg2.int()))
        elif arg1.j_type.value <= 7 and arg2.j_type.value <= 2:
            return J(arg1.date + arg2.int(), arg1.j_type)
        elif arg1.j_type in (JType.DATE, JType.TIMESTAMP) and (
            arg2.j_type in (JType.TIME, JType.DURATION)
        ):
            return J(arg1.nanos_from_epoch() + arg2.data, JType.TIMESTAMP)
        elif (arg1.j_type.value <= 2 and arg2.j_type <= 7) or (
            arg2.j_type == JType.DATE
            and (arg1.j_type == JType.TIME or arg1.j_type == JType.DURATION)
        ):
            return add(arg2, arg1)
        else:
            raise JasmineEvalException(
                "unsupported operand type(s) for '{0}': '{1}' and '{2}'".format(
                    "add", arg1.j_type.name, arg2.j_type.name
                )
            )
