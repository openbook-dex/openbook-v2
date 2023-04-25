from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleHeartbeatParamsJSON(typing.TypedDict):
    permission_bump: int


@dataclass
class OracleHeartbeatParams:
    layout: typing.ClassVar = borsh.CStruct("permission_bump" / borsh.U8)
    permission_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleHeartbeatParams":
        return cls(permission_bump=obj.permission_bump)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"permission_bump": self.permission_bump}

    def to_json(self) -> OracleHeartbeatParamsJSON:
        return {"permission_bump": self.permission_bump}

    @classmethod
    def from_json(cls, obj: OracleHeartbeatParamsJSON) -> "OracleHeartbeatParams":
        return cls(permission_bump=obj["permission_bump"])
