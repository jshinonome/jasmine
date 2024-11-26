import contextlib

with contextlib.suppress(ImportError):
    from jasminum.jasminum import JasmineError, JasmineParseError

__all__ = [JasmineError, JasmineParseError]
