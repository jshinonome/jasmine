import polars.selectors as cs

from .j import J


def selector(column: J) -> J:
    return J(cs.matches(column.to_str()))
