from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class JobInitParamsJSON(typing.TypedDict):
    name: list[int]
    expiration: int
    state_bump: int
    data: list[int]


@dataclass
class JobInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "expiration" / borsh.I64,
        "state_bump" / borsh.U8,
        "data" / borsh.Bytes,
    )
    name: list[int]
    expiration: int
    state_bump: int
    data: bytes

    @classmethod
    def from_decoded(cls, obj: Container) -> "JobInitParams":
        return cls(
            name=obj.name,
            expiration=obj.expiration,
            state_bump=obj.state_bump,
            data=obj.data,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "name": self.name,
            "expiration": self.expiration,
            "state_bump": self.state_bump,
            "data": self.data,
        }

    def to_json(self) -> JobInitParamsJSON:
        return {
            "name": self.name,
            "expiration": self.expiration,
            "state_bump": self.state_bump,
            "data": list(self.data),
        }

    @classmethod
    def from_json(cls, obj: JobInitParamsJSON) -> "JobInitParams":
        return cls(
            name=obj["name"],
            expiration=obj["expiration"],
            state_bump=obj["state_bump"],
            data=bytes(obj["data"]),
        )
