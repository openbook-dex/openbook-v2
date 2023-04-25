from __future__ import annotations
from . import (
    account_meta_zc,
)
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class CallbackZCJSON(typing.TypedDict):
    program_id: str
    accounts: list[account_meta_zc.AccountMetaZCJSON]
    accounts_len: int
    ix_data: list[int]
    ix_data_len: int


@dataclass
class CallbackZC:
    layout: typing.ClassVar = borsh.CStruct(
        "program_id" / BorshPubkey,
        "accounts" / account_meta_zc.AccountMetaZC.layout[32],
        "accounts_len" / borsh.U32,
        "ix_data" / borsh.U8[1024],
        "ix_data_len" / borsh.U32,
    )
    program_id: PublicKey
    accounts: list[account_meta_zc.AccountMetaZC]
    accounts_len: int
    ix_data: list[int]
    ix_data_len: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "CallbackZC":
        return cls(
            program_id=obj.program_id,
            accounts=list(
                map(
                    lambda item: account_meta_zc.AccountMetaZC.from_decoded(item),
                    obj.accounts,
                )
            ),
            accounts_len=obj.accounts_len,
            ix_data=obj.ix_data,
            ix_data_len=obj.ix_data_len,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "program_id": self.program_id,
            "accounts": list(map(lambda item: item.to_encodable(), self.accounts)),
            "accounts_len": self.accounts_len,
            "ix_data": self.ix_data,
            "ix_data_len": self.ix_data_len,
        }

    def to_json(self) -> CallbackZCJSON:
        return {
            "program_id": str(self.program_id),
            "accounts": list(map(lambda item: item.to_json(), self.accounts)),
            "accounts_len": self.accounts_len,
            "ix_data": self.ix_data,
            "ix_data_len": self.ix_data_len,
        }

    @classmethod
    def from_json(cls, obj: CallbackZCJSON) -> "CallbackZC":
        return cls(
            program_id=PublicKey(obj["program_id"]),
            accounts=list(
                map(
                    lambda item: account_meta_zc.AccountMetaZC.from_json(item),
                    obj["accounts"],
                )
            ),
            accounts_len=obj["accounts_len"],
            ix_data=obj["ix_data"],
            ix_data_len=obj["ix_data_len"],
        )
