from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleWithdrawParamsJSON(typing.TypedDict):
    state_bump: int
    permission_bump: int
    amount: int


@dataclass
class OracleWithdrawParams:
    layout: typing.ClassVar = borsh.CStruct(
        "state_bump" / borsh.U8, "permission_bump" / borsh.U8, "amount" / borsh.U64
    )
    state_bump: int
    permission_bump: int
    amount: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleWithdrawParams":
        return cls(
            state_bump=obj.state_bump,
            permission_bump=obj.permission_bump,
            amount=obj.amount,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "state_bump": self.state_bump,
            "permission_bump": self.permission_bump,
            "amount": self.amount,
        }

    def to_json(self) -> OracleWithdrawParamsJSON:
        return {
            "state_bump": self.state_bump,
            "permission_bump": self.permission_bump,
            "amount": self.amount,
        }

    @classmethod
    def from_json(cls, obj: OracleWithdrawParamsJSON) -> "OracleWithdrawParams":
        return cls(
            state_bump=obj["state_bump"],
            permission_bump=obj["permission_bump"],
            amount=obj["amount"],
        )
