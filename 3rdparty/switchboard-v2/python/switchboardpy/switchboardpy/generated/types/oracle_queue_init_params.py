from __future__ import annotations
from . import (
    borsh_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleQueueInitParamsJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    reward: int
    min_stake: int
    feed_probation_period: int
    oracle_timeout: int
    slashing_enabled: bool
    variance_tolerance_multiplier: borsh_decimal.BorshDecimalJSON
    consecutive_feed_failure_limit: int
    consecutive_oracle_failure_limit: int
    queue_size: int
    unpermissioned_feeds: bool
    unpermissioned_vrf: bool


@dataclass
class OracleQueueInitParams:
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[64],
        "reward" / borsh.U64,
        "min_stake" / borsh.U64,
        "feed_probation_period" / borsh.U32,
        "oracle_timeout" / borsh.U32,
        "slashing_enabled" / borsh.Bool,
        "variance_tolerance_multiplier" / borsh_decimal.BorshDecimal.layout,
        "consecutive_feed_failure_limit" / borsh.U64,
        "consecutive_oracle_failure_limit" / borsh.U64,
        "queue_size" / borsh.U32,
        "unpermissioned_feeds" / borsh.Bool,
        "unpermissioned_vrf" / borsh.Bool,
    )
    name: list[int]
    metadata: list[int]
    reward: int
    min_stake: int
    feed_probation_period: int
    oracle_timeout: int
    slashing_enabled: bool
    variance_tolerance_multiplier: borsh_decimal.BorshDecimal
    consecutive_feed_failure_limit: int
    consecutive_oracle_failure_limit: int
    queue_size: int
    unpermissioned_feeds: bool
    unpermissioned_vrf: bool

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleQueueInitParams":
        return cls(
            name=obj.name,
            metadata=obj.metadata,
            reward=obj.reward,
            min_stake=obj.min_stake,
            feed_probation_period=obj.feed_probation_period,
            oracle_timeout=obj.oracle_timeout,
            slashing_enabled=obj.slashing_enabled,
            variance_tolerance_multiplier=borsh_decimal.BorshDecimal.from_decoded(
                obj.variance_tolerance_multiplier
            ),
            consecutive_feed_failure_limit=obj.consecutive_feed_failure_limit,
            consecutive_oracle_failure_limit=obj.consecutive_oracle_failure_limit,
            queue_size=obj.queue_size,
            unpermissioned_feeds=obj.unpermissioned_feeds,
            unpermissioned_vrf=obj.unpermissioned_vrf,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "reward": self.reward,
            "min_stake": self.min_stake,
            "feed_probation_period": self.feed_probation_period,
            "oracle_timeout": self.oracle_timeout,
            "slashing_enabled": self.slashing_enabled,
            "variance_tolerance_multiplier": self.variance_tolerance_multiplier.to_encodable(),
            "consecutive_feed_failure_limit": self.consecutive_feed_failure_limit,
            "consecutive_oracle_failure_limit": self.consecutive_oracle_failure_limit,
            "queue_size": self.queue_size,
            "unpermissioned_feeds": self.unpermissioned_feeds,
            "unpermissioned_vrf": self.unpermissioned_vrf,
        }

    def to_json(self) -> OracleQueueInitParamsJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "reward": self.reward,
            "min_stake": self.min_stake,
            "feed_probation_period": self.feed_probation_period,
            "oracle_timeout": self.oracle_timeout,
            "slashing_enabled": self.slashing_enabled,
            "variance_tolerance_multiplier": self.variance_tolerance_multiplier.to_json(),
            "consecutive_feed_failure_limit": self.consecutive_feed_failure_limit,
            "consecutive_oracle_failure_limit": self.consecutive_oracle_failure_limit,
            "queue_size": self.queue_size,
            "unpermissioned_feeds": self.unpermissioned_feeds,
            "unpermissioned_vrf": self.unpermissioned_vrf,
        }

    @classmethod
    def from_json(cls, obj: OracleQueueInitParamsJSON) -> "OracleQueueInitParams":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            reward=obj["reward"],
            min_stake=obj["min_stake"],
            feed_probation_period=obj["feed_probation_period"],
            oracle_timeout=obj["oracle_timeout"],
            slashing_enabled=obj["slashing_enabled"],
            variance_tolerance_multiplier=borsh_decimal.BorshDecimal.from_json(
                obj["variance_tolerance_multiplier"]
            ),
            consecutive_feed_failure_limit=obj["consecutive_feed_failure_limit"],
            consecutive_oracle_failure_limit=obj["consecutive_oracle_failure_limit"],
            queue_size=obj["queue_size"],
            unpermissioned_feeds=obj["unpermissioned_feeds"],
            unpermissioned_vrf=obj["unpermissioned_vrf"],
        )
