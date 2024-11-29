from typing import Callable

from .ast import AstFn


class JFn:
    fn: Callable | AstFn | None
    args: dict
    arg_names: list[str]
    arg_num: int

    def __init__(
        self,
        fn: Callable | AstFn | None,
        args: dict,
        arg_names: list[str],
        arg_num: int,
    ) -> None:
        self.fn = fn
        self.args = args
        self.arg_names = arg_names
        self.arg_num = arg_num
