from __future__ import annotations
from . import (
    switchboard_decimal,
)
import typing
from dataclasses import dataclass
from construct import Container
from solana.publickey import PublicKey
import borsh_construct as borsh
from anchorpy.borsh_extension import BorshPubkey


class AggregatorRoundJSON(typing.TypedDict):
    num_success: int
    num_error: int
    is_closed: bool
    round_open_slot: int
    round_open_timestamp: int
    result: switchboard_decimal.SwitchboardDecimalJSON
    std_deviation: switchboard_decimal.SwitchboardDecimalJSON
    min_response: switchboard_decimal.SwitchboardDecimalJSON
    max_response: switchboard_decimal.SwitchboardDecimalJSON
    oracle_pubkeys_data: list[str]
    medians_data: list[switchboard_decimal.SwitchboardDecimalJSON]
    current_payout: list[int]
    medians_fulfilled: list[bool]
    errors_fulfilled: list[bool]


@dataclass
class AggregatorRound:
    layout: typing.ClassVar = borsh.CStruct(
        "num_success" / borsh.U32,
        "num_error" / borsh.U32,
        "is_closed" / borsh.Bool,
        "round_open_slot" / borsh.U64,
        "round_open_timestamp" / borsh.I64,
        "result" / switchboard_decimal.SwitchboardDecimal.layout,
        "std_deviation" / switchboard_decimal.SwitchboardDecimal.layout,
        "min_response" / switchboard_decimal.SwitchboardDecimal.layout,
        "max_response" / switchboard_decimal.SwitchboardDecimal.layout,
        "oracle_pubkeys_data" / BorshPubkey[16],
        "medians_data" / switchboard_decimal.SwitchboardDecimal.layout[16],
        "current_payout" / borsh.I64[16],
        "medians_fulfilled" / borsh.Bool[16],
        "errors_fulfilled" / borsh.Bool[16],
    )
    num_success: int
    num_error: int
    is_closed: bool
    round_open_slot: int
    round_open_timestamp: int
    result: switchboard_decimal.SwitchboardDecimal
    std_deviation: switchboard_decimal.SwitchboardDecimal
    min_response: switchboard_decimal.SwitchboardDecimal
    max_response: switchboard_decimal.SwitchboardDecimal
    oracle_pubkeys_data: list[PublicKey]
    medians_data: list[switchboard_decimal.SwitchboardDecimal]
    current_payout: list[int]
    medians_fulfilled: list[bool]
    errors_fulfilled: list[bool]

    @classmethod
    def from_decoded(cls, obj: Container) -> "AggregatorRound":
        return cls(
            num_success=obj.num_success,
            num_error=obj.num_error,
            is_closed=obj.is_closed,
            round_open_slot=obj.round_open_slot,
            round_open_timestamp=obj.round_open_timestamp,
            result=switchboard_decimal.SwitchboardDecimal.from_decoded(obj.result),
            std_deviation=switchboard_decimal.SwitchboardDecimal.from_decoded(
                obj.std_deviation
            ),
            min_response=switchboard_decimal.SwitchboardDecimal.from_decoded(
                obj.min_response
            ),
            max_response=switchboard_decimal.SwitchboardDecimal.from_decoded(
                obj.max_response
            ),
            oracle_pubkeys_data=obj.oracle_pubkeys_data,
            medians_data=list(
                map(
                    lambda item: switchboard_decimal.SwitchboardDecimal.from_decoded(
                        item
                    ),
                    obj.medians_data,
                )
            ),
            current_payout=obj.current_payout,
            medians_fulfilled=obj.medians_fulfilled,
            errors_fulfilled=obj.errors_fulfilled,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "num_success": self.num_success,
            "num_error": self.num_error,
            "is_closed": self.is_closed,
            "round_open_slot": self.round_open_slot,
            "round_open_timestamp": self.round_open_timestamp,
            "result": self.result.to_encodable(),
            "std_deviation": self.std_deviation.to_encodable(),
            "min_response": self.min_response.to_encodable(),
            "max_response": self.max_response.to_encodable(),
            "oracle_pubkeys_data": self.oracle_pubkeys_data,
            "medians_data": list(
                map(lambda item: item.to_encodable(), self.medians_data)
            ),
            "current_payout": self.current_payout,
            "medians_fulfilled": self.medians_fulfilled,
            "errors_fulfilled": self.errors_fulfilled,
        }

    def to_json(self) -> AggregatorRoundJSON:
        return {
            "num_success": self.num_success,
            "num_error": self.num_error,
            "is_closed": self.is_closed,
            "round_open_slot": self.round_open_slot,
            "round_open_timestamp": self.round_open_timestamp,
            "result": self.result.to_json(),
            "std_deviation": self.std_deviation.to_json(),
            "min_response": self.min_response.to_json(),
            "max_response": self.max_response.to_json(),
            "oracle_pubkeys_data": list(
                map(lambda item: str(item), self.oracle_pubkeys_data)
            ),
            "medians_data": list(map(lambda item: item.to_json(), self.medians_data)),
            "current_payout": self.current_payout,
            "medians_fulfilled": self.medians_fulfilled,
            "errors_fulfilled": self.errors_fulfilled,
        }

    @classmethod
    def from_json(cls, obj: AggregatorRoundJSON) -> "AggregatorRound":
        return cls(
            num_success=obj["num_success"],
            num_error=obj["num_error"],
            is_closed=obj["is_closed"],
            round_open_slot=obj["round_open_slot"],
            round_open_timestamp=obj["round_open_timestamp"],
            result=switchboard_decimal.SwitchboardDecimal.from_json(obj["result"]),
            std_deviation=switchboard_decimal.SwitchboardDecimal.from_json(
                obj["std_deviation"]
            ),
            min_response=switchboard_decimal.SwitchboardDecimal.from_json(
                obj["min_response"]
            ),
            max_response=switchboard_decimal.SwitchboardDecimal.from_json(
                obj["max_response"]
            ),
            oracle_pubkeys_data=list(
                map(lambda item: PublicKey(item), obj["oracle_pubkeys_data"])
            ),
            medians_data=list(
                map(
                    lambda item: switchboard_decimal.SwitchboardDecimal.from_json(item),
                    obj["medians_data"],
                )
            ),
            current_payout=obj["current_payout"],
            medians_fulfilled=obj["medians_fulfilled"],
            errors_fulfilled=obj["errors_fulfilled"],
        )
