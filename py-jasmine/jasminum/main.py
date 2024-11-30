from termcolor import colored, cprint

from .context import Context
from .engine import Engine
from .eval import eval_src
from .history_console import HistoryConsole


def main():
    engine = Engine()
    HistoryConsole()
    src = ""
    text = colored("j* ", "magenta")
    while src != "exit":
        try:
            src = input(text)
            engine.sources[0] = (src, "")
            res = eval_src(src, 0, engine, Context(dict()))
            cprint(res, "light_green")
        except EOFError:
            cprint("exit on ctrl+D", "red")
            exit(0)
