from __future__ import annotations
from . import (
    field_element_zc,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class EcvrfIntermediateJSON(typing.TypedDict):
    r: field_element_zc.FieldElementZCJSON
    n_s: field_element_zc.FieldElementZCJSON
    d: field_element_zc.FieldElementZCJSON
    t13: field_element_zc.FieldElementZCJSON
    t15: field_element_zc.FieldElementZCJSON


@dataclass
class EcvrfIntermediate:
    layout: typing.ClassVar = borsh.CStruct(
        "r" / field_element_zc.FieldElementZC.layout,
        "n_s" / field_element_zc.FieldElementZC.layout,
        "d" / field_element_zc.FieldElementZC.layout,
        "t13" / field_element_zc.FieldElementZC.layout,
        "t15" / field_element_zc.FieldElementZC.layout,
    )
    r: field_element_zc.FieldElementZC
    n_s: field_element_zc.FieldElementZC
    d: field_element_zc.FieldElementZC
    t13: field_element_zc.FieldElementZC
    t15: field_element_zc.FieldElementZC

    @classmethod
    def from_decoded(cls, obj: Container) -> "EcvrfIntermediate":
        return cls(
            r=field_element_zc.FieldElementZC.from_decoded(obj.r),
            n_s=field_element_zc.FieldElementZC.from_decoded(obj.n_s),
            d=field_element_zc.FieldElementZC.from_decoded(obj.d),
            t13=field_element_zc.FieldElementZC.from_decoded(obj.t13),
            t15=field_element_zc.FieldElementZC.from_decoded(obj.t15),
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "r": self.r.to_encodable(),
            "n_s": self.n_s.to_encodable(),
            "d": self.d.to_encodable(),
            "t13": self.t13.to_encodable(),
            "t15": self.t15.to_encodable(),
        }

    def to_json(self) -> EcvrfIntermediateJSON:
        return {
            "r": self.r.to_json(),
            "n_s": self.n_s.to_json(),
            "d": self.d.to_json(),
            "t13": self.t13.to_json(),
            "t15": self.t15.to_json(),
        }

    @classmethod
    def from_json(cls, obj: EcvrfIntermediateJSON) -> "EcvrfIntermediate":
        return cls(
            r=field_element_zc.FieldElementZC.from_json(obj["r"]),
            n_s=field_element_zc.FieldElementZC.from_json(obj["n_s"]),
            d=field_element_zc.FieldElementZC.from_json(obj["d"]),
            t13=field_element_zc.FieldElementZC.from_json(obj["t13"]),
            t15=field_element_zc.FieldElementZC.from_json(obj["t15"]),
        )
