from __future__ import annotations
from . import (
    scalar,
    edwards_point_zc,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class EcvrfProofZCJSON(typing.TypedDict):
    gamma: edwards_point_zc.EdwardsPointZCJSON
    c: scalar.ScalarJSON
    s: scalar.ScalarJSON


@dataclass
class EcvrfProofZC:
    layout: typing.ClassVar = borsh.CStruct(
        "gamma" / edwards_point_zc.EdwardsPointZC.layout,
        "c" / scalar.Scalar.layout,
        "s" / scalar.Scalar.layout,
    )
    gamma: edwards_point_zc.EdwardsPointZC
    c: scalar.Scalar
    s: scalar.Scalar

    @classmethod
    def from_decoded(cls, obj: Container) -> "EcvrfProofZC":
        return cls(
            gamma=edwards_point_zc.EdwardsPointZC.from_decoded(obj.gamma),
            c=scalar.Scalar.from_decoded(obj.c),
            s=scalar.Scalar.from_decoded(obj.s),
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "gamma": self.gamma.to_encodable(),
            "c": self.c.to_encodable(),
            "s": self.s.to_encodable(),
        }

    def to_json(self) -> EcvrfProofZCJSON:
        return {
            "gamma": self.gamma.to_json(),
            "c": self.c.to_json(),
            "s": self.s.to_json(),
        }

    @classmethod
    def from_json(cls, obj: EcvrfProofZCJSON) -> "EcvrfProofZC":
        return cls(
            gamma=edwards_point_zc.EdwardsPointZC.from_json(obj["gamma"]),
            c=scalar.Scalar.from_json(obj["c"]),
            s=scalar.Scalar.from_json(obj["s"]),
        )
