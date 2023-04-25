from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSetBatchSizeParamsJSON(typing.TypedDict):
    batch_size: int


@dataclass
class AggregatorSetBatchSizeParams:
    layout: typing.ClassVar = borsh.CStruct("batch_size" / borsh.U32)
    batch_size: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSetBatchSizeParams":
        return cls(batch_size=obj.batch_size)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"batch_size": self.batch_size}

    def to_json(self) -> AggregatorSetBatchSizeParamsJSON:
        return {"batch_size": self.batch_size}

    @classmethod
    def from_json(
        cls, obj: AggregatorSetBatchSizeParamsJSON
    ) -> "AggregatorSetBatchSizeParams":
        return cls(batch_size=obj["batch_size"])
