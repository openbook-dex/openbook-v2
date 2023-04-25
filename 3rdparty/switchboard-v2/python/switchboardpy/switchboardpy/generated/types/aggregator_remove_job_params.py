from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorRemoveJobParamsJSON(typing.TypedDict):
    job_idx: int


@dataclass
class AggregatorRemoveJobParams:
    layout: typing.ClassVar = borsh.CStruct("job_idx" / borsh.U32)
    job_idx: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorRemoveJobParams":
        return cls(job_idx=obj.job_idx)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"job_idx": self.job_idx}

    def to_json(self) -> AggregatorRemoveJobParamsJSON:
        return {"job_idx": self.job_idx}

    @classmethod
    def from_json(
        cls, obj: AggregatorRemoveJobParamsJSON
    ) -> "AggregatorRemoveJobParams":
        return cls(job_idx=obj["job_idx"])
