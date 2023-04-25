from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class CrankPushParamsJSON(typing.TypedDict):
    state_bump: int
    permission_bump: int


@dataclass
class CrankPushParams:
    layout: typing.ClassVar = borsh.CStruct(
        "state_bump" / borsh.U8, "permission_bump" / borsh.U8
    )
    state_bump: int
    permission_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "CrankPushParams":
        return cls(state_bump=obj.state_bump, permission_bump=obj.permission_bump)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"state_bump": self.state_bump, "permission_bump": self.permission_bump}

    def to_json(self) -> CrankPushParamsJSON:
        return {"state_bump": self.state_bump, "permission_bump": self.permission_bump}

    @classmethod
    def from_json(cls, obj: CrankPushParamsJSON) -> "CrankPushParams":
        return cls(state_bump=obj["state_bump"], permission_bump=obj["permission_bump"])
