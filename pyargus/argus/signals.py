from typing import List, Optional, Protocol, Tuple, TypeVar, runtime_checkable

from typing_extensions import Self

from argus._argus import BoolSignal, FloatSignal, IntSignal, UnsignedIntSignal, dtype

T = TypeVar("T", bool, int, float)


@runtime_checkable
class Signal(Protocol[T]):
    def is_empty(self) -> bool:
        ...

    @property
    def start_time(self) -> Optional[float]:
        ...

    @property
    def end_time(self) -> Optional[float]:
        ...

    @property
    def kind(self) -> dtype:
        ...

    @classmethod
    def constant(cls, value: T) -> Self:
        ...

    @classmethod
    def from_samples(cls, samples: List[Tuple[float, T]]) -> Self:
        ...

    def push(self, time: float, value: T) -> None:
        ...

    def at(self, time: float) -> Optional[T]:
        ...


__all__ = [
    "Signal",
    "BoolSignal",
    "IntSignal",
    "UnsignedIntSignal",
    "FloatSignal",
]
