from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleMetricsJSON(typing.TypedDict):
    consecutive_success: int
    consecutive_error: int
    consecutive_disagreement: int
    consecutive_late_response: int
    consecutive_failure: int
    total_success: int
    total_error: int
    total_disagreement: int
    total_late_response: int


@dataclass
class OracleMetrics:
    layout: typing.ClassVar = borsh.CStruct(
        "consecutive_success" / borsh.U64,
        "consecutive_error" / borsh.U64,
        "consecutive_disagreement" / borsh.U64,
        "consecutive_late_response" / borsh.U64,
        "consecutive_failure" / borsh.U64,
        "total_success" / borsh.U128,
        "total_error" / borsh.U128,
        "total_disagreement" / borsh.U128,
        "total_late_response" / borsh.U128,
    )
    consecutive_success: int
    consecutive_error: int
    consecutive_disagreement: int
    consecutive_late_response: int
    consecutive_failure: int
    total_success: int
    total_error: int
    total_disagreement: int
    total_late_response: int

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleMetrics":
        return cls(
            consecutive_success=obj.consecutive_success,
            consecutive_error=obj.consecutive_error,
            consecutive_disagreement=obj.consecutive_disagreement,
            consecutive_late_response=obj.consecutive_late_response,
            consecutive_failure=obj.consecutive_failure,
            total_success=obj.total_success,
            total_error=obj.total_error,
            total_disagreement=obj.total_disagreement,
            total_late_response=obj.total_late_response,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "consecutive_success": self.consecutive_success,
            "consecutive_error": self.consecutive_error,
            "consecutive_disagreement": self.consecutive_disagreement,
            "consecutive_late_response": self.consecutive_late_response,
            "consecutive_failure": self.consecutive_failure,
            "total_success": self.total_success,
            "total_error": self.total_error,
            "total_disagreement": self.total_disagreement,
            "total_late_response": self.total_late_response,
        }

    def to_json(self) -> OracleMetricsJSON:
        return {
            "consecutive_success": self.consecutive_success,
            "consecutive_error": self.consecutive_error,
            "consecutive_disagreement": self.consecutive_disagreement,
            "consecutive_late_response": self.consecutive_late_response,
            "consecutive_failure": self.consecutive_failure,
            "total_success": self.total_success,
            "total_error": self.total_error,
            "total_disagreement": self.total_disagreement,
            "total_late_response": self.total_late_response,
        }

    @classmethod
    def from_json(cls, obj: OracleMetricsJSON) -> "OracleMetrics":
        return cls(
            consecutive_success=obj["consecutive_success"],
            consecutive_error=obj["consecutive_error"],
            consecutive_disagreement=obj["consecutive_disagreement"],
            consecutive_late_response=obj["consecutive_late_response"],
            consecutive_failure=obj["consecutive_failure"],
            total_success=obj["total_success"],
            total_error=obj["total_error"],
            total_disagreement=obj["total_disagreement"],
            total_late_response=obj["total_late_response"],
        )
