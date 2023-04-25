from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class CrankInitParamsJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    crank_size: int


@dataclass
class CrankInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.Bytes, "metadata" / borsh.Bytes, "crank_size" / borsh.U32
    )
    name: bytes
    metadata: bytes
    crank_size: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "CrankInitParams":
        return cls(name=obj.name, metadata=obj.metadata, crank_size=obj.crank_size)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "crank_size": self.crank_size,
        }

    def to_json(self) -> CrankInitParamsJSON:
        return {
            "name": list(self.name),
            "metadata": list(self.metadata),
            "crank_size": self.crank_size,
        }

    @classmethod
    def from_json(cls, obj: CrankInitParamsJSON) -> "CrankInitParams":
        return cls(
            name=bytes(obj["name"]),
            metadata=bytes(obj["metadata"]),
            crank_size=obj["crank_size"],
        )
