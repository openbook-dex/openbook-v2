from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class LeaseExtendParamsJSON(typing.TypedDict):
    load_amount: int
    lease_bump: int
    state_bump: int


@dataclass
class LeaseExtendParams:
    layout: typing.ClassVar = borsh.CStruct(
        "load_amount" / borsh.U64, "lease_bump" / borsh.U8, "state_bump" / borsh.U8
    )
    load_amount: int
    lease_bump: int
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "LeaseExtendParams":
        return cls(
            load_amount=obj.load_amount,
            lease_bump=obj.lease_bump,
            state_bump=obj.state_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "load_amount": self.load_amount,
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    def to_json(self) -> LeaseExtendParamsJSON:
        return {
            "load_amount": self.load_amount,
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    @classmethod
    def from_json(cls, obj: LeaseExtendParamsJSON) -> "LeaseExtendParams":
        return cls(
            load_amount=obj["load_amount"],
            lease_bump=obj["lease_bump"],
            state_bump=obj["state_bump"],
        )
