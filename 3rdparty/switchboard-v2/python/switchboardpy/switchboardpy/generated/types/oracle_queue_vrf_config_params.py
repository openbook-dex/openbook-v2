from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class OracleQueueVrfConfigParamsJSON(typing.TypedDict):
    unpermissioned_vrf_enabled: bool


@dataclass
class OracleQueueVrfConfigParams:
    layout: typing.ClassVar = borsh.CStruct("unpermissioned_vrf_enabled" / borsh.Bool)
    unpermissioned_vrf_enabled: bool

    @classmethod
    def from_decoded(cls, obj: Container) -> "OracleQueueVrfConfigParams":
        return cls(unpermissioned_vrf_enabled=obj.unpermissioned_vrf_enabled)

    def to_encodable(self) -> dict[str, typing.Any]:
        return {"unpermissioned_vrf_enabled": self.unpermissioned_vrf_enabled}

    def to_json(self) -> OracleQueueVrfConfigParamsJSON:
        return {"unpermissioned_vrf_enabled": self.unpermissioned_vrf_enabled}

    @classmethod
    def from_json(
        cls, obj: OracleQueueVrfConfigParamsJSON
    ) -> "OracleQueueVrfConfigParams":
        return cls(unpermissioned_vrf_enabled=obj["unpermissioned_vrf_enabled"])
