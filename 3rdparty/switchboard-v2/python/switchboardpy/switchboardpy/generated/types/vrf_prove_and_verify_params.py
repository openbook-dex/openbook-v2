from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfProveAndVerifyParamsJSON(typing.TypedDict):
    nonce: typing.Optional[int]
    state_bump: int
    idx: int
    proof: list[int]


@dataclass
class VrfProveAndVerifyParams:
    layout: typing.ClassVar = borsh.CStruct(
        "nonce" / borsh.Option(borsh.U32),
        "state_bump" / borsh.U8,
        "idx" / borsh.U32,
        "proof" / borsh.Bytes,
    )
    nonce: typing.Optional[int]
    state_bump: int
    idx: int
    proof: bytes

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfProveAndVerifyParams":
        return cls(
            nonce=obj.nonce, state_bump=obj.state_bump, idx=obj.idx, proof=obj.proof
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "nonce": self.nonce,
            "state_bump": self.state_bump,
            "idx": self.idx,
            "proof": self.proof,
        }

    def to_json(self) -> VrfProveAndVerifyParamsJSON:
        return {
            "nonce": self.nonce,
            "state_bump": self.state_bump,
            "idx": self.idx,
            "proof": list(self.proof),
        }

    @classmethod
    def from_json(cls, obj: VrfProveAndVerifyParamsJSON) -> "VrfProveAndVerifyParams":
        return cls(
            nonce=obj["nonce"],
            state_bump=obj["state_bump"],
            idx=obj["idx"],
            proof=bytes(obj["proof"]),
        )
