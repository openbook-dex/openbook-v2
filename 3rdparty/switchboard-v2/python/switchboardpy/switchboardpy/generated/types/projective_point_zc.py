from __future__ import annotations
from . import (
    field_element_zc,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class ProjectivePointZCJSON(typing.TypedDict):
    x: field_element_zc.FieldElementZCJSON
    y: field_element_zc.FieldElementZCJSON
    z: field_element_zc.FieldElementZCJSON


@dataclass
class ProjectivePointZC:
    layout: typing.ClassVar = borsh.CStruct(
        "x" / field_element_zc.FieldElementZC.layout,
        "y" / field_element_zc.FieldElementZC.layout,
        "z" / field_element_zc.FieldElementZC.layout,
    )
    x: field_element_zc.FieldElementZC
    y: field_element_zc.FieldElementZC
    z: field_element_zc.FieldElementZC

    @classmethod
    def from_decoded(cls, obj: Container) -> "ProjectivePointZC":
        return cls(
            x=field_element_zc.FieldElementZC.from_decoded(obj.x),
            y=field_element_zc.FieldElementZC.from_decoded(obj.y),
            z=field_element_zc.FieldElementZC.from_decoded(obj.z),
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "x": self.x.to_encodable(),
            "y": self.y.to_encodable(),
            "z": self.z.to_encodable(),
        }

    def to_json(self) -> ProjectivePointZCJSON:
        return {"x": self.x.to_json(), "y": self.y.to_json(), "z": self.z.to_json()}

    @classmethod
    def from_json(cls, obj: ProjectivePointZCJSON) -> "ProjectivePointZC":
        return cls(
            x=field_element_zc.FieldElementZC.from_json(obj["x"]),
            y=field_element_zc.FieldElementZC.from_json(obj["y"]),
            z=field_element_zc.FieldElementZC.from_json(obj["z"]),
        )
