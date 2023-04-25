from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class LeaseWithdrawParamsJSON(typing.TypedDict):
    state_bump: int
    lease_bump: int
    amount: int


@dataclass
class LeaseWithdrawParams:
    layout: typing.ClassVar = borsh.CStruct(
        "state_bump" / borsh.U8, "lease_bump" / borsh.U8, "amount" / borsh.U64
    )
    state_bump: int
    lease_bump: int
    amount: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "LeaseWithdrawParams":
        return cls(
            state_bump=obj.state_bump, lease_bump=obj.lease_bump, amount=obj.amount
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "state_bump": self.state_bump,
            "lease_bump": self.lease_bump,
            "amount": self.amount,
        }

    def to_json(self) -> LeaseWithdrawParamsJSON:
        return {
            "state_bump": self.state_bump,
            "lease_bump": self.lease_bump,
            "amount": self.amount,
        }

    @classmethod
    def from_json(cls, obj: LeaseWithdrawParamsJSON) -> "LeaseWithdrawParams":
        return cls(
            state_bump=obj["state_bump"],
            lease_bump=obj["lease_bump"],
            amount=obj["amount"],
        )
