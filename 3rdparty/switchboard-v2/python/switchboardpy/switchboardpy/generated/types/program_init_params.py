from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class ProgramInitParamsJSON(typing.TypedDict):
    state_bump: int


@dataclass
class ProgramInitParams:
    layout: typing.ClassVar = borsh.CStruct("state_bump" / borsh.U8)
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "ProgramInitParams":
        return cls(state_bump=obj.state_bump)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"state_bump": self.state_bump}

    def to_json(self) -> ProgramInitParamsJSON:
        return {"state_bump": self.state_bump}

    @classmethod
    def from_json(cls, obj: ProgramInitParamsJSON) -> "ProgramInitParams":
        return cls(state_bump=obj["state_bump"])
