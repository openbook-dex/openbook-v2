from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
from anchorpy.borsh_extension import BorshPubkey

import borsh_construct as borsh


class LeaseInitParamsJSON(typing.TypedDict):
    load_amount: int
    withdraw_authority: str
    lease_bump: int
    state_bump: int


@dataclass
class LeaseInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "load_amount" / borsh.U64,
        "withdraw_authority" / BorshPubkey,
        "lease_bump" / borsh.U8,
        "state_bump" / borsh.U8,
    )
    load_amount: int
    withdraw_authority: PublicKey
    lease_bump: int
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "LeaseInitParams":
        return cls(
            load_amount=obj.load_amount,
            withdraw_authority=obj.withdraw_authority,
            lease_bump=obj.lease_bump,
            state_bump=obj.state_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "load_amount": self.load_amount,
            "withdraw_authority": self.withdraw_authority,
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    def to_json(self) -> LeaseInitParamsJSON:
        return {
            "load_amount": self.load_amount,
            "withdraw_authority": str(self.withdraw_authority),
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    @classmethod
    def from_json(cls, obj: LeaseInitParamsJSON) -> "LeaseInitParams":
        return cls(
            load_amount=obj["load_amount"],
            withdraw_authority=PublicKey(obj["withdraw_authority"]),
            lease_bump=obj["lease_bump"],
            state_bump=obj["state_bump"],
        )
