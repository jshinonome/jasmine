from .j import J, JType
from .j_fn import JFn


def add(arg1: J, arg2: J):
    if isinstance(arg1, J) and isinstance(arg2, J):
        return J(arg1.data + arg2.data, JType.INT)
