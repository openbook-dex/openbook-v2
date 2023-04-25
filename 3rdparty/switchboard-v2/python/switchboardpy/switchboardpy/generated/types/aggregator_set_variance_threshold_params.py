from __future__ import annotations
from . import (
    borsh_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class AggregatorSetVarianceThresholdParamsJSON(typing.TypedDict):
    variance_threshold: borsh_decimal.BorshDecimalJSON


@dataclass
class AggregatorSetVarianceThresholdParams:
    layout: typing.ClassVar = borsh.CStruct(
        "variance_threshold" / borsh_decimal.BorshDecimal.layout
    )
    variance_threshold: borsh_decimal.BorshDecimal

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorSetVarianceThresholdParams":
        return cls(
            variance_threshold=borsh_decimal.BorshDecimal.from_decoded(
                obj.variance_threshold
            )
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"variance_threshold": self.variance_threshold.to_encodable()}

    def to_json(self) -> AggregatorSetVarianceThresholdParamsJSON:
        return {"variance_threshold": self.variance_threshold.to_json()}

    @classmethod
    def from_json(
        cls, obj: AggregatorSetVarianceThresholdParamsJSON
    ) -> "AggregatorSetVarianceThresholdParams":
        return cls(
            variance_threshold=borsh_decimal.BorshDecimal.from_json(
                obj["variance_threshold"]
            )
        )
