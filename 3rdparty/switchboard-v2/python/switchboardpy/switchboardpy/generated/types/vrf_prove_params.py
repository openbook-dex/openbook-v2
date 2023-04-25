from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfProveParamsJSON(typing.TypedDict):
    proof: list[int]
    idx: int


@dataclass
class VrfProveParams:
    layout: typing.ClassVar = borsh.CStruct("proof" / borsh.Bytes, "idx" / borsh.U32)
    proof: bytes
    idx: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfProveParams":
        return cls(proof=obj.proof, idx=obj.idx)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"proof": self.proof, "idx": self.idx}

    def to_json(self) -> VrfProveParamsJSON:
        return {"proof": list(self.proof), "idx": self.idx}

    @classmethod
    def from_json(cls, obj: VrfProveParamsJSON) -> "VrfProveParams":
        return cls(proof=bytes(obj["proof"]), idx=obj["idx"])
