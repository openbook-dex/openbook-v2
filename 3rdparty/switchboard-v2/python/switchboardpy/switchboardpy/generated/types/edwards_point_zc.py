from __future__ import annotations
from . import (
    field_element_zc,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class EdwardsPointZCJSON(typing.TypedDict):
    x: field_element_zc.FieldElementZCJSON
    y: field_element_zc.FieldElementZCJSON
    z: field_element_zc.FieldElementZCJSON
    t: field_element_zc.FieldElementZCJSON


@dataclass
class EdwardsPointZC:
    layout: typing.ClassVar = borsh.CStruct(
        "x" / field_element_zc.FieldElementZC.layout,
        "y" / field_element_zc.FieldElementZC.layout,
        "z" / field_element_zc.FieldElementZC.layout,
        "t" / field_element_zc.FieldElementZC.layout,
    )
    x: field_element_zc.FieldElementZC
    y: field_element_zc.FieldElementZC
    z: field_element_zc.FieldElementZC
    t: field_element_zc.FieldElementZC

    @classmethod
    def from_decoded(cls, obj: Container) -> "EdwardsPointZC":
        return cls(
            x=field_element_zc.FieldElementZC.from_decoded(obj.x),
            y=field_element_zc.FieldElementZC.from_decoded(obj.y),
            z=field_element_zc.FieldElementZC.from_decoded(obj.z),
            t=field_element_zc.FieldElementZC.from_decoded(obj.t),
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "x": self.x.to_encodable(),
            "y": self.y.to_encodable(),
            "z": self.z.to_encodable(),
            "t": self.t.to_encodable(),
        }

    def to_json(self) -> EdwardsPointZCJSON:
        return {
            "x": self.x.to_json(),
            "y": self.y.to_json(),
            "z": self.z.to_json(),
            "t": self.t.to_json(),
        }

    @classmethod
    def from_json(cls, obj: EdwardsPointZCJSON) -> "EdwardsPointZC":
        return cls(
            x=field_element_zc.FieldElementZC.from_json(obj["x"]),
            y=field_element_zc.FieldElementZC.from_json(obj["y"]),
            z=field_element_zc.FieldElementZC.from_json(obj["z"]),
            t=field_element_zc.FieldElementZC.from_json(obj["t"]),
        )
