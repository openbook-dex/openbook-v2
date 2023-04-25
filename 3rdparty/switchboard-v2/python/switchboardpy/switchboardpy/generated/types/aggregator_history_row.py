from __future__ import annotations
from . import (
    switchboard_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorHistoryRowJSON(typing.TypedDict):
    timestamp: int
    value: switchboard_decimal.SwitchboardDecimalJSON


@dataclass
class AggregatorHistoryRow:
    layout: typing.ClassVar = borsh.CStruct(
        "timestamp" / borsh.I64, "value" / switchboard_decimal.SwitchboardDecimal.layout
    )
    timestamp: int
    value: switchboard_decimal.SwitchboardDecimal

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorHistoryRow":
        return cls(
            timestamp=obj.timestamp,
            value=switchboard_decimal.SwitchboardDecimal.from_decoded(obj.value),
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"timestamp": self.timestamp, "value": self.value.to_encodable()}

    def to_json(self) -> AggregatorHistoryRowJSON:
        return {"timestamp": self.timestamp, "value": self.value.to_json()}

    @classmethod
    def from_json(cls, obj: AggregatorHistoryRowJSON) -> "AggregatorHistoryRow":
        return cls(
            timestamp=obj["timestamp"],
            value=switchboard_decimal.SwitchboardDecimal.from_json(obj["value"]),
        )
