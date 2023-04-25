from __future__ import annotations
import typing
from dataclasses import dataclass
from anchorpy.borsh_extension import EnumForCodegen
import borsh_construct as borsh


class AAAAJSON(typing.TypedDict):
    kind: typing.Literal["AAAA"]


class BBBBJSON(typing.TypedDict):
    kind: typing.Literal["BBBB"]


class BADCJSON(typing.TypedDict):
    kind: typing.Literal["BADC"]


class BACDJSON(typing.TypedDict):
    kind: typing.Literal["BACD"]


class ADDAJSON(typing.TypedDict):
    kind: typing.Literal["ADDA"]


class CBCBJSON(typing.TypedDict):
    kind: typing.Literal["CBCB"]


class ABDCJSON(typing.TypedDict):
    kind: typing.Literal["ABDC"]


class ABABJSON(typing.TypedDict):
    kind: typing.Literal["ABAB"]


class DBBDJSON(typing.TypedDict):
    kind: typing.Literal["DBBD"]


class CACAJSON(typing.TypedDict):
    kind: typing.Literal["CACA"]


@dataclass
class AAAA:
    discriminator: typing.ClassVar = 0
    kind: typing.ClassVar = "AAAA"

    @classmethod
    def to_json(cls) -> AAAAJSON:
        return AAAAJSON(
            kind="AAAA",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "AAAA": {},
        }


@dataclass
class BBBB:
    discriminator: typing.ClassVar = 1
    kind: typing.ClassVar = "BBBB"

    @classmethod
    def to_json(cls) -> BBBBJSON:
        return BBBBJSON(
            kind="BBBB",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "BBBB": {},
        }


@dataclass
class BADC:
    discriminator: typing.ClassVar = 2
    kind: typing.ClassVar = "BADC"

    @classmethod
    def to_json(cls) -> BADCJSON:
        return BADCJSON(
            kind="BADC",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "BADC": {},
        }


@dataclass
class BACD:
    discriminator: typing.ClassVar = 3
    kind: typing.ClassVar = "BACD"

    @classmethod
    def to_json(cls) -> BACDJSON:
        return BACDJSON(
            kind="BACD",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "BACD": {},
        }


@dataclass
class ADDA:
    discriminator: typing.ClassVar = 4
    kind: typing.ClassVar = "ADDA"

    @classmethod
    def to_json(cls) -> ADDAJSON:
        return ADDAJSON(
            kind="ADDA",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "ADDA": {},
        }


@dataclass
class CBCB:
    discriminator: typing.ClassVar = 5
    kind: typing.ClassVar = "CBCB"

    @classmethod
    def to_json(cls) -> CBCBJSON:
        return CBCBJSON(
            kind="CBCB",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "CBCB": {},
        }


@dataclass
class ABDC:
    discriminator: typing.ClassVar = 6
    kind: typing.ClassVar = "ABDC"

    @classmethod
    def to_json(cls) -> ABDCJSON:
        return ABDCJSON(
            kind="ABDC",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "ABDC": {},
        }


@dataclass
class ABAB:
    discriminator: typing.ClassVar = 7
    kind: typing.ClassVar = "ABAB"

    @classmethod
    def to_json(cls) -> ABABJSON:
        return ABABJSON(
            kind="ABAB",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "ABAB": {},
        }


@dataclass
class DBBD:
    discriminator: typing.ClassVar = 8
    kind: typing.ClassVar = "DBBD"

    @classmethod
    def to_json(cls) -> DBBDJSON:
        return DBBDJSON(
            kind="DBBD",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "DBBD": {},
        }


@dataclass
class CACA:
    discriminator: typing.ClassVar = 9
    kind: typing.ClassVar = "CACA"

    @classmethod
    def to_json(cls) -> CACAJSON:
        return CACAJSON(
            kind="CACA",
        )

    @classmethod
    def to_encodable(cls) -> dict:
        return {
            "CACA": {},
        }


ShuffleKind = typing.Union[AAAA, BBBB, BADC, BACD, ADDA, CBCB, ABDC, ABAB, DBBD, CACA]
ShuffleJSON = typing.Union[
    AAAAJSON,
    BBBBJSON,
    BADCJSON,
    BACDJSON,
    ADDAJSON,
    CBCBJSON,
    ABDCJSON,
    ABABJSON,
    DBBDJSON,
    CACAJSON,
]


def from_decoded(obj: dict) -> ShuffleKind:
    if not isinstance(obj, dict):
        raise ValueError("Invalid enum object")
    if "AAAA" in obj:
        return AAAA()
    if "BBBB" in obj:
        return BBBB()
    if "BADC" in obj:
        return BADC()
    if "BACD" in obj:
        return BACD()
    if "ADDA" in obj:
        return ADDA()
    if "CBCB" in obj:
        return CBCB()
    if "ABDC" in obj:
        return ABDC()
    if "ABAB" in obj:
        return ABAB()
    if "DBBD" in obj:
        return DBBD()
    if "CACA" in obj:
        return CACA()
    raise ValueError("Invalid enum object")


def from_json(obj: ShuffleJSON) -> ShuffleKind:
    if obj["kind"] == "AAAA":
        return AAAA()
    if obj["kind"] == "BBBB":
        return BBBB()
    if obj["kind"] == "BADC":
        return BADC()
    if obj["kind"] == "BACD":
        return BACD()
    if obj["kind"] == "ADDA":
        return ADDA()
    if obj["kind"] == "CBCB":
        return CBCB()
    if obj["kind"] == "ABDC":
        return ABDC()
    if obj["kind"] == "ABAB":
        return ABAB()
    if obj["kind"] == "DBBD":
        return DBBD()
    if obj["kind"] == "CACA":
        return CACA()
    kind = obj["kind"]
    raise ValueError(f"Unrecognized enum kind: {kind}")


layout = EnumForCodegen(
    "AAAA" / borsh.CStruct(),
    "BBBB" / borsh.CStruct(),
    "BADC" / borsh.CStruct(),
    "BACD" / borsh.CStruct(),
    "ADDA" / borsh.CStruct(),
    "CBCB" / borsh.CStruct(),
    "ABDC" / borsh.CStruct(),
    "ABAB" / borsh.CStruct(),
    "DBBD" / borsh.CStruct(),
    "CACA" / borsh.CStruct(),
)
