from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfRoundJSON(typing.TypedDict):
    alpha: list[int]
    alpha_len: int
    request_slot: int
    request_timestamp: int
    result: list[int]
    num_verified: int
    ebuf: list[int]


@dataclass
class VrfRound:
    layout: typing.ClassVar = borsh.CStruct(
        "alpha" / borsh.U8[256],
        "alpha_len" / borsh.U32,
        "request_slot" / borsh.U64,
        "request_timestamp" / borsh.I64,
        "result" / borsh.U8[32],
        "num_verified" / borsh.U32,
        "ebuf" / borsh.U8[256],
    )
    alpha: list[int]
    alpha_len: int
    request_slot: int
    request_timestamp: int
    result: list[int]
    num_verified: int
    ebuf: list[int]

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfRound":
        return cls(
            alpha=obj.alpha,
            alpha_len=obj.alpha_len,
            request_slot=obj.request_slot,
            request_timestamp=obj.request_timestamp,
            result=obj.result,
            num_verified=obj.num_verified,
            ebuf=obj.ebuf,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "alpha": self.alpha,
            "alpha_len": self.alpha_len,
            "request_slot": self.request_slot,
            "request_timestamp": self.request_timestamp,
            "result": self.result,
            "num_verified": self.num_verified,
            "ebuf": self.ebuf,
        }

    def to_json(self) -> VrfRoundJSON:
        return {
            "alpha": self.alpha,
            "alpha_len": self.alpha_len,
            "request_slot": self.request_slot,
            "request_timestamp": self.request_timestamp,
            "result": self.result,
            "num_verified": self.num_verified,
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: VrfRoundJSON) -> "VrfRound":
        return cls(
            alpha=obj["alpha"],
            alpha_len=obj["alpha_len"],
            request_slot=obj["request_slot"],
            request_timestamp=obj["request_timestamp"],
            result=obj["result"],
            num_verified=obj["num_verified"],
            ebuf=obj["ebuf"],
        )
