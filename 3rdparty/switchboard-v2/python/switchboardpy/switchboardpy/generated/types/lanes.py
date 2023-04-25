from __future__ import annotations
import typing
from dataclasses import dataclass
from anchorpy.borsh_extension import EnumForCodegen
import borsh_construct as borsh


class DJSON(typing.TypedDict):
    kind: typing.Literal["D"]


class CJSON(typing.TypedDict):
    kind: typing.Literal["C"]


class ABJSON(typing.TypedDict):
    kind: typing.Literal["AB"]


class ACJSON(typing.TypedDict):
    kind: typing.Literal["AC"]


class ADJSON(typing.TypedDict):
    kind: typing.Literal["AD"]


class BCDJSON(typing.TypedDict):
    kind: typing.Literal["BCD"]


@dataclass
class D:
    discriminator: typing.ClassVar = 0
    kind: typing.ClassVar = "D"

    @classmethod
    def to_json(cls) -> DJSON:
        return DJSON(
            kind="D",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "D": {},
        }


@dataclass
class C:
    discriminator: typing.ClassVar = 1
    kind: typing.ClassVar = "C"

    @classmethod
    def to_json(cls) -> CJSON:
        return CJSON(
            kind="C",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "C": {},
        }


@dataclass
class AB:
    discriminator: typing.ClassVar = 2
    kind: typing.ClassVar = "AB"

    @classmethod
    def to_json(cls) -> ABJSON:
        return ABJSON(
            kind="AB",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "AB": {},
        }


@dataclass
class AC:
    discriminator: typing.ClassVar = 3
    kind: typing.ClassVar = "AC"

    @classmethod
    def to_json(cls) -> ACJSON:
        return ACJSON(
            kind="AC",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "AC": {},
        }


@dataclass
class AD:
    discriminator: typing.ClassVar = 4
    kind: typing.ClassVar = "AD"

    @classmethod
    def to_json(cls) -> ADJSON:
        return ADJSON(
            kind="AD",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "AD": {},
        }


@dataclass
class BCD:
    discriminator: typing.ClassVar = 5
    kind: typing.ClassVar = "BCD"

    @classmethod
    def to_json(cls) -> BCDJSON:
        return BCDJSON(
            kind="BCD",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "BCD": {},
        }


LanesKind = typing.Union[D, C, AB, AC, AD, BCD]
LanesJSON = typing.Union[DJSON, CJSON, ABJSON, ACJSON, ADJSON, BCDJSON]


def from_decoded(obj: dict) -> LanesKind:
    if not isinstance(obj, dict):
        raise ValueError("Invalid enum object")
    if "D" in obj:
        return D()
    if "C" in obj:
        return C()
    if "AB" in obj:
        return AB()
    if "AC" in obj:
        return AC()
    if "AD" in obj:
        return AD()
    if "BCD" in obj:
        return BCD()
    raise ValueError("Invalid enum object")


def from_json(obj: LanesJSON) -> LanesKind:
    if obj["kind"] == "D":
        return D()
    if obj["kind"] == "C":
        return C()
    if obj["kind"] == "AB":
        return AB()
    if obj["kind"] == "AC":
        return AC()
    if obj["kind"] == "AD":
        return AD()
    if obj["kind"] == "BCD":
        return BCD()
    kind = obj["kind"]
    raise ValueError(f"Unrecognized enum kind: {kind}")


layout = EnumForCodegen(
    "D" / borsh.CStruct(),
    "C" / borsh.CStruct(),
    "AB" / borsh.CStruct(),
    "AC" / borsh.CStruct(),
    "AD" / borsh.CStruct(),
    "BCD" / borsh.CStruct(),
)
