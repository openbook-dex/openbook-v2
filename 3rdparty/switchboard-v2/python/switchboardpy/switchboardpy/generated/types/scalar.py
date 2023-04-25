from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class ScalarJSON(typing.TypedDict):
    bytes: list[int]


@dataclass
class Scalar:
    layout: typing.ClassVar = borsh.CStruct("bytes" / borsh.U8[32])
    bytes: list[int]

    @classmethod
    def from_decoded(cls, obj: Container) -> "Scalar":
        return cls(bytes=obj.bytes)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"bytes": self.bytes}

    def to_json(self) -> ScalarJSON:
        return {"bytes": self.bytes}

    @classmethod
    def from_json(cls, obj: ScalarJSON) -> "Scalar":
        return cls(bytes=obj["bytes"])
