from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class AccountMetaZCJSON(typing.TypedDict):
    pubkey: str
    is_signer: bool
    is_writable: bool


@dataclass
class AccountMetaZC:
    layout: typing.ClassVar = borsh.CStruct(
        "pubkey" / BorshPubkey, "is_signer" / borsh.Bool, "is_writable" / borsh.Bool
    )
    pubkey: PublicKey
    is_signer: bool
    is_writable: bool

    @classmethod
    def from_decoded(cls, obj: Container) -> "AccountMetaZC":
        return cls(
            pubkey=obj.pubkey, is_signer=obj.is_signer, is_writable=obj.is_writable
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "pubkey": self.pubkey,
            "is_signer": self.is_signer,
            "is_writable": self.is_writable,
        }

    def to_json(self) -> AccountMetaZCJSON:
        return {
            "pubkey": str(self.pubkey),
            "is_signer": self.is_signer,
            "is_writable": self.is_writable,
        }

    @classmethod
    def from_json(cls, obj: AccountMetaZCJSON) -> "AccountMetaZC":
        return cls(
            pubkey=PublicKey(obj["pubkey"]),
            is_signer=obj["is_signer"],
            is_writable=obj["is_writable"],
        )
