from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSetUpdateIntervalParamsJSON(typing.TypedDict):
    new_interval: int


@dataclass
class AggregatorSetUpdateIntervalParams:
    layout: typing.ClassVar = borsh.CStruct("new_interval" / borsh.U32)
    new_interval: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSetUpdateIntervalParams":
        return cls(new_interval=obj.new_interval)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"new_interval": self.new_interval}

    def to_json(self) -> AggregatorSetUpdateIntervalParamsJSON:
        return {"new_interval": self.new_interval}

    @classmethod
    def from_json(
        cls, obj: AggregatorSetUpdateIntervalParamsJSON
    ) -> "AggregatorSetUpdateIntervalParams":
        return cls(new_interval=obj["new_interval"])
