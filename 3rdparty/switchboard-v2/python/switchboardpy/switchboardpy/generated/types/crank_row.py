from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class CrankRowJSON(typing.TypedDict):
    pubkey: str
    next_timestamp: int


@dataclass
class CrankRow:
    layout: typing.ClassVar = borsh.CStruct(
        "pubkey" / BorshPubkey, "next_timestamp" / borsh.I64
    )
    pubkey: PublicKey
    next_timestamp: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "CrankRow":
        return cls(pubkey=obj.pubkey, next_timestamp=obj.next_timestamp)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"pubkey": self.pubkey, "next_timestamp": self.next_timestamp}

    def to_json(self) -> CrankRowJSON:
        return {"pubkey": str(self.pubkey), "next_timestamp": self.next_timestamp}

    @classmethod
    def from_json(cls, obj: CrankRowJSON) -> "CrankRow":
        return cls(
            pubkey=PublicKey(obj["pubkey"]), next_timestamp=obj["next_timestamp"]
        )
