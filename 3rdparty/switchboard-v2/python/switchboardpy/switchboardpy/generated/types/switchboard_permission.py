from __future__ import annotations
import typing
from dataclasses import dataclass
from anchorpy.borsh_extension import EnumForCodegen
import borsh_construct as borsh


class PermitOracleHeartbeatJSON(typing.TypedDict):
    kind: typing.Literal["PermitOracleHeartbeat"]


class PermitOracleQueueUsageJSON(typing.TypedDict):
    kind: typing.Literal["PermitOracleQueueUsage"]


class PermitVrfRequestsJSON(typing.TypedDict):
    kind: typing.Literal["PermitVrfRequests"]


@dataclass
class PermitOracleHeartbeat:
    discriminator: typing.ClassVar = 0
    kind: typing.ClassVar = "PermitOracleHeartbeat"

    @classmethod
    def to_json(cls) -> PermitOracleHeartbeatJSON:
        return PermitOracleHeartbeatJSON(
            kind="PermitOracleHeartbeat",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "PermitOracleHeartbeat": {},
        }


@dataclass
class PermitOracleQueueUsage:
    discriminator: typing.ClassVar = 1
    kind: typing.ClassVar = "PermitOracleQueueUsage"

    @classmethod
    def to_json(cls) -> PermitOracleQueueUsageJSON:
        return PermitOracleQueueUsageJSON(
            kind="PermitOracleQueueUsage",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "PermitOracleQueueUsage": {},
        }


@dataclass
class PermitVrfRequests:
    discriminator: typing.ClassVar = 2
    kind: typing.ClassVar = "PermitVrfRequests"

    @classmethod
    def to_json(cls) -> PermitVrfRequestsJSON:
        return PermitVrfRequestsJSON(
            kind="PermitVrfRequests",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "PermitVrfRequests": {},
        }


SwitchboardPermissionKind = typing.Union[
    PermitOracleHeartbeat, PermitOracleQueueUsage, PermitVrfRequests
]
SwitchboardPermissionJSON = typing.Union[
    PermitOracleHeartbeatJSON, PermitOracleQueueUsageJSON, PermitVrfRequestsJSON
]


def from_decoded(obj: dict) -> SwitchboardPermissionKind:
    if not isinstance(obj, dict):
        raise ValueError("Invalid enum object")
    if "PermitOracleHeartbeat" in obj:
        return PermitOracleHeartbeat()
    if "PermitOracleQueueUsage" in obj:
        return PermitOracleQueueUsage()
    if "PermitVrfRequests" in obj:
        return PermitVrfRequests()
    raise ValueError("Invalid enum object")


def from_json(obj: SwitchboardPermissionJSON) -> SwitchboardPermissionKind:
    if obj["kind"] == "PermitOracleHeartbeat":
        return PermitOracleHeartbeat()
    if obj["kind"] == "PermitOracleQueueUsage":
        return PermitOracleQueueUsage()
    if obj["kind"] == "PermitVrfRequests":
        return PermitVrfRequests()
    kind = obj["kind"]
    raise ValueError(f"Unrecognized enum kind: {kind}")


layout = EnumForCodegen(
    "PermitOracleHeartbeat" / borsh.CStruct(),
    "PermitOracleQueueUsage" / borsh.CStruct(),
    "PermitVrfRequests" / borsh.CStruct(),
)
