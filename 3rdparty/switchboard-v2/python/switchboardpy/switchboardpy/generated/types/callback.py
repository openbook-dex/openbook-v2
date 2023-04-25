from __future__ import annotations
from . import (
    account_meta_borsh,
)
import typing
from dataclasses import dataclass
from construct import Container, Construct
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class CallbackJSON(typing.TypedDict):
    program_id: str
    accounts: list[account_meta_borsh.AccountMetaBorshJSON]
    ix_data: list[int]


@dataclass
class Callback:
    layout: typing.ClassVar = borsh.CStruct(
        "program_id" / BorshPubkey,
        "accounts"
        / borsh.Vec(typing.cast(Construct, account_meta_borsh.AccountMetaBorsh.layout)),
        "ix_data" / borsh.Bytes,
    )
    program_id: PublicKey
    accounts: list[account_meta_borsh.AccountMetaBorsh]
    ix_data: bytes

    @classmethod
    def from_decoded(cls, obj: Container) -> "Callback":
        return cls(
            program_id=obj.program_id,
            accounts=list(
                map(
                    lambda item: account_meta_borsh.AccountMetaBorsh.from_decoded(item),
                    obj.accounts,
                )
            ),
            ix_data=obj.ix_data,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "program_id": self.program_id,
            "accounts": list(map(lambda item: item.to_encodable(), self.accounts)),
            "ix_data": self.ix_data,
        }

    def to_json(self) -> CallbackJSON:
        return {
            "program_id": str(self.program_id),
            "accounts": list(map(lambda item: item.to_json(), self.accounts)),
            "ix_data": list(self.ix_data),
        }

    @classmethod
    def from_json(cls, obj: CallbackJSON) -> "Callback":
        return cls(
            program_id=PublicKey(obj["program_id"]),
            accounts=list(
                map(
                    lambda item: account_meta_borsh.AccountMetaBorsh.from_json(item),
                    obj["accounts"],
                )
            ),
            ix_data=bytes(obj["ix_data"]),
        )
