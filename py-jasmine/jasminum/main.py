import importlib.metadata
import readline
import traceback

from termcolor import cprint

from .context import Context
from .engine import Engine
from .eval import SQL_FN, eval_src
from .history_console import HistoryConsole

__version__ = importlib.metadata.version("jasminum")


def main():
    print(
        "\x1b[1;32m\
    \n\
         ▄█    ▄████████    ▄████████   ▄▄▄▄███▄▄▄▄    ▄█  ███▄▄▄▄   \n\
        ███   ███    ███   ███    ███ ▄██▀▀▀███▀▀▀██▄ ███  ███▀▀▀██▄ \n\
        ███   ███    ███   ███    █▀  ███   ███   ███ ███▌ ███   ███ \n\
        ███   ███    ███   ███        ███   ███   ███ ███▌ ███   ███ \n\
        ███ ▀███████████ ▀███████████ ███   ███   ███ ███▌ ███   ███ \n\
        ███   ███    ███          ███ ███   ███   ███ ███  ███   ███ \n\
        ███   ███    ███    ▄█    ███ ███   ███   ███ ███  ███   ███ \n\
    █▄ ▄███   ███    █▀   ▄████████▀   ▀█   ███   █▀  █▀    ▀█   █▀  \n\
    ▀▀▀▀▀▀                                                           ver {} \x1b[0m\n".format(
            __version__
        )
    )

    engine = Engine()
    HistoryConsole()
    src = ""
    readline.set_completer(complete)
    while src != "exit":
        try:
            src = []
            line = input("j*  ")
            if line == "":
                continue
            else:
                src.append(line)
            while True:
                line = input("*   ")
                if not line:
                    break
                src.append(line)
            src = "\n".join(src)
            engine.sources[0] = (src, "")
            try:
                res = eval_src(src, 0, engine, Context(dict()))
                cprint(res, "light_green")
            except Exception as e:
                traceback.print_exc()
                cprint(e, "red")
        except EOFError:
            cprint("exit on ctrl+D", "red")
            exit(0)
        except KeyboardInterrupt:
            print()
            continue


def complete(text, state):
    for cmd in SQL_FN:
        if cmd.startswith(text):
            if not state:
                return cmd
            else:
                state -= 1
