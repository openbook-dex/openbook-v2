from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class FieldElementZCJSON(typing.TypedDict):
    bytes: list[int]


@dataclass
class FieldElementZC:
    layout: typing.ClassVar = borsh.CStruct("bytes" / borsh.U64[5])
    bytes: list[int]

    @classmethod
    def from_decoded(cls, obj: Container) -> "FieldElementZC":
        return cls(bytes=obj.bytes)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"bytes": self.bytes}

    def to_json(self) -> FieldElementZCJSON:
        return {"bytes": self.bytes}

    @classmethod
    def from_json(cls, obj: FieldElementZCJSON) -> "FieldElementZC":
        return cls(bytes=obj["bytes"])
