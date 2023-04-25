from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSetMinOraclesParamsJSON(typing.TypedDict):
    min_oracle_results: int


@dataclass
class AggregatorSetMinOraclesParams:
    layout: typing.ClassVar = borsh.CStruct("min_oracle_results" / borsh.U32)
    min_oracle_results: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSetMinOraclesParams":
        return cls(min_oracle_results=obj.min_oracle_results)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"min_oracle_results": self.min_oracle_results}

    def to_json(self) -> AggregatorSetMinOraclesParamsJSON:
        return {"min_oracle_results": self.min_oracle_results}

    @classmethod
    def from_json(
        cls, obj: AggregatorSetMinOraclesParamsJSON
    ) -> "AggregatorSetMinOraclesParams":
        return cls(min_oracle_results=obj["min_oracle_results"])
