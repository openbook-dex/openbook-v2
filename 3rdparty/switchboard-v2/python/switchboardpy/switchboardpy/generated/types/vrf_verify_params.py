from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfVerifyParamsJSON(typing.TypedDict):
    nonce: typing.Optional[int]
    state_bump: int
    idx: int


@dataclass
class VrfVerifyParams:
    layout: typing.ClassVar = borsh.CStruct(
        "nonce" / borsh.Option(borsh.U32), "state_bump" / borsh.U8, "idx" / borsh.U32
    )
    nonce: typing.Optional[int]
    state_bump: int
    idx: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfVerifyParams":
        return cls(nonce=obj.nonce, state_bump=obj.state_bump, idx=obj.idx)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"nonce": self.nonce, "state_bump": self.state_bump, "idx": self.idx}

    def to_json(self) -> VrfVerifyParamsJSON:
        return {"nonce": self.nonce, "state_bump": self.state_bump, "idx": self.idx}

    @classmethod
    def from_json(cls, obj: VrfVerifyParamsJSON) -> "VrfVerifyParams":
        return cls(nonce=obj["nonce"], state_bump=obj["state_bump"], idx=obj["idx"])
