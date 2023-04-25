from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleInitParamsJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    state_bump: int
    oracle_bump: int


@dataclass
class OracleInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.Bytes,
        "metadata" / borsh.Bytes,
        "state_bump" / borsh.U8,
        "oracle_bump" / borsh.U8,
    )
    name: bytes
    metadata: bytes
    state_bump: int
    oracle_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleInitParams":
        return cls(
            name=obj.name,
            metadata=obj.metadata,
            state_bump=obj.state_bump,
            oracle_bump=obj.oracle_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "state_bump": self.state_bump,
            "oracle_bump": self.oracle_bump,
        }

    def to_json(self) -> OracleInitParamsJSON:
        return {
            "name": list(self.name),
            "metadata": list(self.metadata),
            "state_bump": self.state_bump,
            "oracle_bump": self.oracle_bump,
        }

    @classmethod
    def from_json(cls, obj: OracleInitParamsJSON) -> "OracleInitParams":
        return cls(
            name=bytes(obj["name"]),
            metadata=bytes(obj["metadata"]),
            state_bump=obj["state_bump"],
            oracle_bump=obj["oracle_bump"],
        )
