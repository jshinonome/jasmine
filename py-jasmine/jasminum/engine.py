import zoneinfo
from pathlib import Path
from typing import Callable

import polars as pl

from .ast import print_trace
from .exceptions import JasmineEvalException
from .io import wpart
from .j import J, JParted, JType
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
        self.register_builtin("load", lambda x: self.load_partitioned_df(x))
        self.register_builtin("wpart", wpart)
        self.builtins["tz"] = J(
            pl.Series("tz", sorted(list(zoneinfo.available_timezones())))
        )

    def register_builtin(self, name: str, fn: Callable) -> None:
        arg_num = fn.__code__.co_argcount
        self.builtins[name] = JFn(
            fn,
            dict(),
            list(fn.__code__.co_varnames[:arg_num]),
            arg_num,
        )

    def get_trace(self, source_id: int, pos: int, msg: str) -> str:
        source, path = self.sources.get(source_id)
        return print_trace(source, path, pos, msg)

    # YYYYMMDD_00
    # YYYY_00
    def load_partitioned_df(self, path: J) -> J:
        if path.j_type != JType.CAT and path.j_type != JType.STRING:
            raise JasmineEvalException(
                "'load' requires cat|string, got %s" % path.j_type
            )
        p = Path(path.data).resolve()
        frames = []
        for df_path in p.iterdir():
            # skip name starts with digit
            if df_path.name[0].isdigit():
                continue
            else:
                if df_path.is_file():
                    self.globals[df_path.name] = J(JParted(df_path, 0, []))
                    frames.append(df_path.name)
                else:
                    partitions = []
                    unit = 0
                    for partition in df_path.iterdir():
                        if unit == 0:
                            if len(partition.name) <= 8:
                                unit = 4
                            else:
                                unit = 8
                        partitions.append(int(partition.name[:unit]))
                    if len(partitions) > 0:
                        self.globals[df_path.name] = J(
                            JParted(df_path, unit, sorted(partitions))
                        )
                        frames.append(df_path.name)
        return J(pl.Series("", frames))
