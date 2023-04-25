from __future__ import annotations
from . import (
    switchboard_permission,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class PermissionSetParamsJSON(typing.TypedDict):
    permission: switchboard_permission.SwitchboardPermissionJSON
    enable: bool


@dataclass
class PermissionSetParams:
    layout: typing.ClassVar = borsh.CStruct(
        "permission" / switchboard_permission.layout, "enable" / borsh.Bool
    )
    permission: switchboard_permission.SwitchboardPermissionKind
    enable: bool

    @classmethod
    def from_decoded(cls, obj: Container) -> "PermissionSetParams":
        return cls(
            permission=switchboard_permission.from_decoded(obj.permission),
            enable=obj.enable,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"permission": self.permission.to_encodable(), "enable": self.enable}

    def to_json(self) -> PermissionSetParamsJSON:
        return {"permission": self.permission.to_json(), "enable": self.enable}

    @classmethod
    def from_json(cls, obj: PermissionSetParamsJSON) -> "PermissionSetParams":
        return cls(
            permission=switchboard_permission.from_json(obj["permission"]),
            enable=obj["enable"],
        )
