from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorOpenRoundParamsJSON(typing.TypedDict):
    state_bump: int
    lease_bump: int
    permission_bump: int
    jitter: int


@dataclass
class AggregatorOpenRoundParams:
    layout: typing.ClassVar = borsh.CStruct(
        "state_bump" / borsh.U8,
        "lease_bump" / borsh.U8,
        "permission_bump" / borsh.U8,
        "jitter" / borsh.U8,
    )
    state_bump: int
    lease_bump: int
    permission_bump: int
    jitter: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorOpenRoundParams":
        return cls(
            state_bump=obj.state_bump,
            lease_bump=obj.lease_bump,
            permission_bump=obj.permission_bump,
            jitter=obj.jitter,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "state_bump": self.state_bump,
            "lease_bump": self.lease_bump,
            "permission_bump": self.permission_bump,
            "jitter": self.jitter,
        }

    def to_json(self) -> AggregatorOpenRoundParamsJSON:
        return {
            "state_bump": self.state_bump,
            "lease_bump": self.lease_bump,
            "permission_bump": self.permission_bump,
            "jitter": self.jitter,
        }

    @classmethod
    def from_json(
        cls, obj: AggregatorOpenRoundParamsJSON
    ) -> "AggregatorOpenRoundParams":
        return cls(
            state_bump=obj["state_bump"],
            lease_bump=obj["lease_bump"],
            permission_bump=obj["permission_bump"],
            jitter=obj["jitter"],
        )
