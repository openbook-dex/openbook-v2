from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class HashJSON(typing.TypedDict):
    data: list[int]


@dataclass
class Hash:
    layout: typing.ClassVar = borsh.CStruct("data" / borsh.U8[32])
    data: list[int]

    @classmethod
    def from_decoded(cls, obj: Container) -> "Hash":
        return cls(data=obj.data)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"data": self.data}

    def to_json(self) -> HashJSON:
        return {"data": self.data}

    @classmethod
    def from_json(cls, obj: HashJSON) -> "Hash":
        return cls(data=obj["data"])
