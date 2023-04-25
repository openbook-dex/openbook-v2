from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class BorshDecimalJSON(typing.TypedDict):
    mantissa: int
    scale: int


@dataclass
class BorshDecimal:
    layout: typing.ClassVar = borsh.CStruct(
        "mantissa" / borsh.I128, "scale" / borsh.U32
    )
    mantissa: int
    scale: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "BorshDecimal":
        return cls(mantissa=obj.mantissa, scale=obj.scale)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"mantissa": self.mantissa, "scale": self.scale}

    def to_json(self) -> BorshDecimalJSON:
        return {"mantissa": self.mantissa, "scale": self.scale}

    @classmethod
    def from_json(cls, obj: BorshDecimalJSON) -> "BorshDecimal":
        return cls(mantissa=obj["mantissa"], scale=obj["scale"])
