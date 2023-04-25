from __future__ import annotations
from . import (
    borsh_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSaveResultParamsJSON(typing.TypedDict):
    oracle_idx: int
    error: bool
    value: borsh_decimal.BorshDecimalJSON
    jobs_checksum: list[int]
    min_response: borsh_decimal.BorshDecimalJSON
    max_response: borsh_decimal.BorshDecimalJSON
    feed_permission_bump: int
    oracle_permission_bump: int
    lease_bump: int
    state_bump: int


@dataclass
class AggregatorSaveResultParams:
    layout: typing.ClassVar = borsh.CStruct(
        "oracle_idx" / borsh.U32,
        "error" / borsh.Bool,
        "value" / borsh_decimal.BorshDecimal.layout,
        "jobs_checksum" / borsh.U8[32],
        "min_response" / borsh_decimal.BorshDecimal.layout,
        "max_response" / borsh_decimal.BorshDecimal.layout,
        "feed_permission_bump" / borsh.U8,
        "oracle_permission_bump" / borsh.U8,
        "lease_bump" / borsh.U8,
        "state_bump" / borsh.U8,
    )
    oracle_idx: int
    error: bool
    value: borsh_decimal.BorshDecimal
    jobs_checksum: list[int]
    min_response: borsh_decimal.BorshDecimal
    max_response: borsh_decimal.BorshDecimal
    feed_permission_bump: int
    oracle_permission_bump: int
    lease_bump: int
    state_bump: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSaveResultParams":
        return cls(
            oracle_idx=obj.oracle_idx,
            error=obj.error,
            value=borsh_decimal.BorshDecimal.from_decoded(obj.value),
            jobs_checksum=obj.jobs_checksum,
            min_response=borsh_decimal.BorshDecimal.from_decoded(obj.min_response),
            max_response=borsh_decimal.BorshDecimal.from_decoded(obj.max_response),
            feed_permission_bump=obj.feed_permission_bump,
            oracle_permission_bump=obj.oracle_permission_bump,
            lease_bump=obj.lease_bump,
            state_bump=obj.state_bump,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "oracle_idx": self.oracle_idx,
            "error": self.error,
            "value": self.value.to_encodable(),
            "jobs_checksum": self.jobs_checksum,
            "min_response": self.min_response.to_encodable(),
            "max_response": self.max_response.to_encodable(),
            "feed_permission_bump": self.feed_permission_bump,
            "oracle_permission_bump": self.oracle_permission_bump,
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    def to_json(self) -> AggregatorSaveResultParamsJSON:
        return {
            "oracle_idx": self.oracle_idx,
            "error": self.error,
            "value": self.value.to_json(),
            "jobs_checksum": self.jobs_checksum,
            "min_response": self.min_response.to_json(),
            "max_response": self.max_response.to_json(),
            "feed_permission_bump": self.feed_permission_bump,
            "oracle_permission_bump": self.oracle_permission_bump,
            "lease_bump": self.lease_bump,
            "state_bump": self.state_bump,
        }

    @classmethod
    def from_json(
        cls, obj: AggregatorSaveResultParamsJSON
    ) -> "AggregatorSaveResultParams":
        return cls(
            oracle_idx=obj["oracle_idx"],
            error=obj["error"],
            value=borsh_decimal.BorshDecimal.from_json(obj["value"]),
            jobs_checksum=obj["jobs_checksum"],
            min_response=borsh_decimal.BorshDecimal.from_json(obj["min_response"]),
            max_response=borsh_decimal.BorshDecimal.from_json(obj["max_response"]),
            feed_permission_bump=obj["feed_permission_bump"],
            oracle_permission_bump=obj["oracle_permission_bump"],
            lease_bump=obj["lease_bump"],
            state_bump=obj["state_bump"],
        )
