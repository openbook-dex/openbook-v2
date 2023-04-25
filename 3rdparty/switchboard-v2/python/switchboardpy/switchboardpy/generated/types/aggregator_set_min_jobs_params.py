from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSetMinJobsParamsJSON(typing.TypedDict):
    min_job_results: int


@dataclass
class AggregatorSetMinJobsParams:
    layout: typing.ClassVar = borsh.CStruct("min_job_results" / borsh.U32)
    min_job_results: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSetMinJobsParams":
        return cls(min_job_results=obj.min_job_results)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"min_job_results": self.min_job_results}

    def to_json(self) -> AggregatorSetMinJobsParamsJSON:
        return {"min_job_results": self.min_job_results}

    @classmethod
    def from_json(
        cls, obj: AggregatorSetMinJobsParamsJSON
    ) -> "AggregatorSetMinJobsParams":
        return cls(min_job_results=obj["min_job_results"])
