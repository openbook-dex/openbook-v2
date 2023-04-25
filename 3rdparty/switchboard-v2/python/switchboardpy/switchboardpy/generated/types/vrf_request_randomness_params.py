from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfRequestRandomnessParamsJSON(typing.TypedDict):
    permission_bump: int
    state_bump: int


@dataclass
class VrfRequestRandomnessParams:
    layout: typing.ClassVar = borsh.CStruct(
        "permission_bump" / borsh.U8, "state_bump" / borsh.U8
    )
    permission_bump: int
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfRequestRandomnessParams":
        return cls(permission_bump=obj.permission_bump, state_bump=obj.state_bump)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"permission_bump": self.permission_bump, "state_bump": self.state_bump}

    def to_json(self) -> VrfRequestRandomnessParamsJSON:
        return {"permission_bump": self.permission_bump, "state_bump": self.state_bump}

    @classmethod
    def from_json(
        cls, obj: VrfRequestRandomnessParamsJSON
    ) -> "VrfRequestRandomnessParams":
        return cls(permission_bump=obj["permission_bump"], state_bump=obj["state_bump"])
