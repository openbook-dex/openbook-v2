from __future__ import annotations
from . import (
    callback,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class VrfInitParamsJSON(typing.TypedDict):
    callback: callback.CallbackJSON
    state_bump: int


@dataclass
class VrfInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "callback" / callback.Callback.layout, "state_bump" / borsh.U8
    )
    callback: callback.Callback
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "VrfInitParams":
        return cls(
            callback=callback.Callback.from_decoded(obj.callback),
            state_bump=obj.state_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"callback": self.callback.to_encodable(), "state_bump": self.state_bump}

    def to_json(self) -> VrfInitParamsJSON:
        return {"callback": self.callback.to_json(), "state_bump": self.state_bump}

    @classmethod
    def from_json(cls, obj: VrfInitParamsJSON) -> "VrfInitParams":
        return cls(
            callback=callback.Callback.from_json(obj["callback"]),
            state_bump=obj["state_bump"],
        )
