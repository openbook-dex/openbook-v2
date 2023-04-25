from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleQueueSetRewardsParamsJSON(typing.TypedDict):
    rewards: int


@dataclass
class OracleQueueSetRewardsParams:
    layout: typing.ClassVar = borsh.CStruct("rewards" / borsh.U64)
    rewards: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleQueueSetRewardsParams":
        return cls(rewards=obj.rewards)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"rewards": self.rewards}

    def to_json(self) -> OracleQueueSetRewardsParamsJSON:
        return {"rewards": self.rewards}

    @classmethod
    def from_json(
        cls, obj: OracleQueueSetRewardsParamsJSON
    ) -> "OracleQueueSetRewardsParams":
        return cls(rewards=obj["rewards"])
