from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class ProgramConfigParamsJSON(typing.TypedDict):
    token: str
    bump: int


@dataclass
class ProgramConfigParams:
    layout: typing.ClassVar = borsh.CStruct("token" / BorshPubkey, "bump" / borsh.U8)
    token: PublicKey
    bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "ProgramConfigParams":
        return cls(token=obj.token, bump=obj.bump)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"token": self.token, "bump": self.bump}

    def to_json(self) -> ProgramConfigParamsJSON:
        return {"token": str(self.token), "bump": self.bump}

    @classmethod
    def from_json(cls, obj: ProgramConfigParamsJSON) -> "ProgramConfigParams":
        return cls(token=PublicKey(obj["token"]), bump=obj["bump"])
