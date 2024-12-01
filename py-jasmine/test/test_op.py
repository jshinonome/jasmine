import pytest

from jasminum.context import Context
from jasminum.engine import Engine
from jasminum.eval import eval_src
from jasminum.j import J, JType


@pytest.mark.parametrize(
    "src,expect",
    [
        ("1+1", J(2, JType.INT)),
        ("2024-10-23+00:12:34.5", J(1729642354500000000, JType.TIMESTAMP)),
    ],
)
def test_simple_src(src, expect):
    engine = Engine()
    res = eval_src(src, 0, engine, Context(dict()))
    assert res == expect
