import polars as pl

from .j import J, JType


def tz(datetime: J, tz: J) -> J:
    tzinfo = tz.to_str()
    datetime.assert_types([JType.DATETIME, JType.TIMESTAMP, JType.SERIES])
    if datetime.j_type == JType.DATETIME or datetime.j_type == JType.TIMESTAMP:
        return datetime.with_timezone(tzinfo)
    elif datetime.j_type == JType.SERIES:
        return J(datetime.data.dt.convert_time_zone(tzinfo))
