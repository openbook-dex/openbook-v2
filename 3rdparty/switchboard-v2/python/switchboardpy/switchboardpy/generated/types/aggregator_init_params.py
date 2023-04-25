from __future__ import annotations
from . import (
    borsh_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorInitParamsJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    batch_size: int
    min_oracle_results: int
    min_job_results: int
    min_update_delay_seconds: int
    start_after: int
    variance_threshold: borsh_decimal.BorshDecimalJSON
    force_report_period: int
    expiration: int
    state_bump: int


@dataclass
class AggregatorInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[128],
        "batch_size" / borsh.U32,
        "min_oracle_results" / borsh.U32,
        "min_job_results" / borsh.U32,
        "min_update_delay_seconds" / borsh.U32,
        "start_after" / borsh.I64,
        "variance_threshold" / borsh_decimal.BorshDecimal.layout,
        "force_report_period" / borsh.I64,
        "expiration" / borsh.I64,
        "state_bump" / borsh.U8,
    )
    name: list[int]
    metadata: list[int]
    batch_size: int
    min_oracle_results: int
    min_job_results: int
    min_update_delay_seconds: int
    start_after: int
    variance_threshold: borsh_decimal.BorshDecimal
    force_report_period: int
    expiration: int
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorInitParams":
        return cls(
            name=obj.name,
            metadata=obj.metadata,
            batch_size=obj.batch_size,
            min_oracle_results=obj.min_oracle_results,
            min_job_results=obj.min_job_results,
            min_update_delay_seconds=obj.min_update_delay_seconds,
            start_after=obj.start_after,
            variance_threshold=borsh_decimal.BorshDecimal.from_decoded(
                obj.variance_threshold
            ),
            force_report_period=obj.force_report_period,
            expiration=obj.expiration,
            state_bump=obj.state_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "batch_size": self.batch_size,
            "min_oracle_results": self.min_oracle_results,
            "min_job_results": self.min_job_results,
            "min_update_delay_seconds": self.min_update_delay_seconds,
            "start_after": self.start_after,
            "variance_threshold": self.variance_threshold.to_encodable(),
            "force_report_period": self.force_report_period,
            "expiration": self.expiration,
            "state_bump": self.state_bump,
        }

    def to_json(self) -> AggregatorInitParamsJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "batch_size": self.batch_size,
            "min_oracle_results": self.min_oracle_results,
            "min_job_results": self.min_job_results,
            "min_update_delay_seconds": self.min_update_delay_seconds,
            "start_after": self.start_after,
            "variance_threshold": self.variance_threshold.to_json(),
            "force_report_period": self.force_report_period,
            "expiration": self.expiration,
            "state_bump": self.state_bump,
        }

    @classmethod
    def from_json(cls, obj: AggregatorInitParamsJSON) -> "AggregatorInitParams":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            batch_size=obj["batch_size"],
            min_oracle_results=obj["min_oracle_results"],
            min_job_results=obj["min_job_results"],
            min_update_delay_seconds=obj["min_update_delay_seconds"],
            start_after=obj["start_after"],
            variance_threshold=borsh_decimal.BorshDecimal.from_json(
                obj["variance_threshold"]
            ),
            force_report_period=obj["force_report_period"],
            expiration=obj["expiration"],
            state_bump=obj["state_bump"],
        )
