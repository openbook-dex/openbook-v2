from __future__ import annotations
import typing
from dataclasses import dataclass
from anchorpy.borsh_extension import EnumForCodegen
import borsh_construct as borsh


class StatusNoneJSON(typing.TypedDict):
    kind: typing.Literal["StatusNone"]


class StatusRequestingJSON(typing.TypedDict):
    kind: typing.Literal["StatusRequesting"]


class StatusVerifyingJSON(typing.TypedDict):
    kind: typing.Literal["StatusVerifying"]


class StatusVerifiedJSON(typing.TypedDict):
    kind: typing.Literal["StatusVerified"]


class StatusCallbackSuccessJSON(typing.TypedDict):
    kind: typing.Literal["StatusCallbackSuccess"]


class StatusVerifyFailureJSON(typing.TypedDict):
    kind: typing.Literal["StatusVerifyFailure"]


@dataclass
class StatusNone:
    discriminator: typing.ClassVar = 0
    kind: typing.ClassVar = "StatusNone"

    @classmethod
    def to_json(cls) -> StatusNoneJSON:
        return StatusNoneJSON(
            kind="StatusNone",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusNone": {},
        }


@dataclass
class StatusRequesting:
    discriminator: typing.ClassVar = 1
    kind: typing.ClassVar = "StatusRequesting"

    @classmethod
    def to_json(cls) -> StatusRequestingJSON:
        return StatusRequestingJSON(
            kind="StatusRequesting",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusRequesting": {},
        }


@dataclass
class StatusVerifying:
    discriminator: typing.ClassVar = 2
    kind: typing.ClassVar = "StatusVerifying"

    @classmethod
    def to_json(cls) -> StatusVerifyingJSON:
        return StatusVerifyingJSON(
            kind="StatusVerifying",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusVerifying": {},
        }


@dataclass
class StatusVerified:
    discriminator: typing.ClassVar = 3
    kind: typing.ClassVar = "StatusVerified"

    @classmethod
    def to_json(cls) -> StatusVerifiedJSON:
        return StatusVerifiedJSON(
            kind="StatusVerified",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusVerified": {},
        }


@dataclass
class StatusCallbackSuccess:
    discriminator: typing.ClassVar = 4
    kind: typing.ClassVar = "StatusCallbackSuccess"

    @classmethod
    def to_json(cls) -> StatusCallbackSuccessJSON:
        return StatusCallbackSuccessJSON(
            kind="StatusCallbackSuccess",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusCallbackSuccess": {},
        }


@dataclass
class StatusVerifyFailure:
    discriminator: typing.ClassVar = 5
    kind: typing.ClassVar = "StatusVerifyFailure"

    @classmethod
    def to_json(cls) -> StatusVerifyFailureJSON:
        return StatusVerifyFailureJSON(
            kind="StatusVerifyFailure",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "StatusVerifyFailure": {},
        }


VrfStatusKind = typing.Union[
    StatusNone,
    StatusRequesting,
    StatusVerifying,
    StatusVerified,
    StatusCallbackSuccess,
    StatusVerifyFailure,
]
VrfStatusJSON = typing.Union[
    StatusNoneJSON,
    StatusRequestingJSON,
    StatusVerifyingJSON,
    StatusVerifiedJSON,
    StatusCallbackSuccessJSON,
    StatusVerifyFailureJSON,
]


def from_decoded(obj: dict) -> VrfStatusKind:
    if not isinstance(obj, dict):
        raise ValueError("Invalid enum object")
    if "StatusNone" in obj:
        return StatusNone()
    if "StatusRequesting" in obj:
        return StatusRequesting()
    if "StatusVerifying" in obj:
        return StatusVerifying()
    if "StatusVerified" in obj:
        return StatusVerified()
    if "StatusCallbackSuccess" in obj:
        return StatusCallbackSuccess()
    if "StatusVerifyFailure" in obj:
        return StatusVerifyFailure()
    raise ValueError("Invalid enum object")


def from_json(obj: VrfStatusJSON) -> VrfStatusKind:
    if obj["kind"] == "StatusNone":
        return StatusNone()
    if obj["kind"] == "StatusRequesting":
        return StatusRequesting()
    if obj["kind"] == "StatusVerifying":
        return StatusVerifying()
    if obj["kind"] == "StatusVerified":
        return StatusVerified()
    if obj["kind"] == "StatusCallbackSuccess":
        return StatusCallbackSuccess()
    if obj["kind"] == "StatusVerifyFailure":
        return StatusVerifyFailure()
    kind = obj["kind"]
    raise ValueError(f"Unrecognized enum kind: {kind}")


layout = EnumForCodegen(
    "StatusNone" / borsh.CStruct(),
    "StatusRequesting" / borsh.CStruct(),
    "StatusVerifying" / borsh.CStruct(),
    "StatusVerified" / borsh.CStruct(),
    "StatusCallbackSuccess" / borsh.CStruct(),
    "StatusVerifyFailure" / borsh.CStruct(),
)
