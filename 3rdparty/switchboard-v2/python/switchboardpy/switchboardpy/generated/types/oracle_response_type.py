from __future__ import annotations
import typing
from dataclasses import dataclass
from anchorpy.borsh_extension import EnumForCodegen
import borsh_construct as borsh


class TypeSuccessJSON(typing.TypedDict):
    kind: typing.Literal["TypeSuccess"]


class TypeErrorJSON(typing.TypedDict):
    kind: typing.Literal["TypeError"]


class TypeDisagreementJSON(typing.TypedDict):
    kind: typing.Literal["TypeDisagreement"]


class TypeNoResponseJSON(typing.TypedDict):
    kind: typing.Literal["TypeNoResponse"]


@dataclass
class TypeSuccess:
    discriminator: typing.ClassVar = 0
    kind: typing.ClassVar = "TypeSuccess"

    @classmethod
    def to_json(cls) -> TypeSuccessJSON:
        return TypeSuccessJSON(
            kind="TypeSuccess",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "TypeSuccess": {},
        }


@dataclass
class TypeError:
    discriminator: typing.ClassVar = 1
    kind: typing.ClassVar = "TypeError"

    @classmethod
    def to_json(cls) -> TypeErrorJSON:
        return TypeErrorJSON(
            kind="TypeError",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "TypeError": {},
        }


@dataclass
class TypeDisagreement:
    discriminator: typing.ClassVar = 2
    kind: typing.ClassVar = "TypeDisagreement"

    @classmethod
    def to_json(cls) -> TypeDisagreementJSON:
        return TypeDisagreementJSON(
            kind="TypeDisagreement",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "TypeDisagreement": {},
        }


@dataclass
class TypeNoResponse:
    discriminator: typing.ClassVar = 3
    kind: typing.ClassVar = "TypeNoResponse"

    @classmethod
    def to_json(cls) -> TypeNoResponseJSON:
        return TypeNoResponseJSON(
            kind="TypeNoResponse",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "TypeNoResponse": {},
        }


OracleResponseTypeKind = typing.Union[
    TypeSuccess, TypeError, TypeDisagreement, TypeNoResponse
]
OracleResponseTypeJSON = typing.Union[
    TypeSuccessJSON, TypeErrorJSON, TypeDisagreementJSON, TypeNoResponseJSON
]


def from_decoded(obj: dict) -> OracleResponseTypeKind:
    if not isinstance(obj, dict):
        raise ValueError("Invalid enum object")
    if "TypeSuccess" in obj:
        return TypeSuccess()
    if "TypeError" in obj:
        return TypeError()
    if "TypeDisagreement" in obj:
        return TypeDisagreement()
    if "TypeNoResponse" in obj:
        return TypeNoResponse()
    raise ValueError("Invalid enum object")


def from_json(obj: OracleResponseTypeJSON) -> OracleResponseTypeKind:
    if obj["kind"] == "TypeSuccess":
        return TypeSuccess()
    if obj["kind"] == "TypeError":
        return TypeError()
    if obj["kind"] == "TypeDisagreement":
        return TypeDisagreement()
    if obj["kind"] == "TypeNoResponse":
        return TypeNoResponse()
    kind = obj["kind"]
    raise ValueError(f"Unrecognized enum kind: {kind}")


layout = EnumForCodegen(
    "TypeSuccess" / borsh.CStruct(),
    "TypeError" / borsh.CStruct(),
    "TypeDisagreement" / borsh.CStruct(),
    "TypeNoResponse" / borsh.CStruct(),
)
