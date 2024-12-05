import zoneinfo
from typing import Callable

import polars as pl

from .ast import print_trace
from .j import J
from .j_fn import JFn
from .operator import add, cast, rand


class Engine:
    globals: dict[str, any]
    handles: dict[int, any]
    sources: dict[int, (str, str)]
    builtins: dict[str, any]

    def __init__(self) -> None:
        self.globals = dict()
        self.handles = dict()
        self.sources = dict()
        self.builtins = dict()

        self.register_builtin("+", add)
        self.register_builtin("?", rand)
        self.register_builtin("$", cast)
        self.builtins["tz"] = J(
            pl.Series("tz", sorted(list(zoneinfo.available_timezones())))
        )

    def register_builtin(self, name: str, fn: Callable) -> None:
        self.builtins[name] = JFn(
            fn,
            dict(),
            list(fn.__code__.co_varnames),
            fn.__code__.co_argcount,
        )

    def get_trace(self, source_id: int, pos: int, msg: str) -> str:
        source, path = self.sources.get(source_id)
        return print_trace(source, path, pos, msg)
