from typing import Callable

from .j_fn import JFn
from .operator import add


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

    def register_builtin(self, name: str, fn: Callable) -> None:
        self.builtins[name] = JFn(
            fn,
            dict(),
            list(fn.__code__.co_varnames),
            fn.__code__.co_argcount,
        )