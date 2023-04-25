from __future__ import annotations
import typing
from dataclasses import dataclass
from construct import Container
import borsh_construct as borsh


class CrankPopParamsJSON(typing.TypedDict):
    state_bump: int
    lease_bumps: list[int]
    permission_bumps: list[int]
    nonce: typing.Optional[int]
    fail_open_on_account_mismatch: typing.Optional[bool]


@dataclass
class CrankPopParams:
    layout: typing.ClassVar = borsh.CStruct(
        "state_bump" / borsh.U8,
        "lease_bumps" / borsh.Bytes,
        "permission_bumps" / borsh.Bytes,
        "nonce" / borsh.Option(borsh.U32),
        "fail_open_on_account_mismatch" / borsh.Option(borsh.Bool),
    )
    state_bump: int
    lease_bumps: bytes
    permission_bumps: bytes
    nonce: typing.Optional[int]
    fail_open_on_account_mismatch: typing.Optional[bool]

    @classmethod
    def from_decoded(cls, obj: Container) -> "CrankPopParams":
        return cls(
            state_bump=obj.state_bump,
            lease_bumps=obj.lease_bumps,
            permission_bumps=obj.permission_bumps,
            nonce=obj.nonce,
            fail_open_on_account_mismatch=obj.fail_open_on_account_mismatch,
        )

    def to_encodable(self) -> dict[str, typing.Any]:
        return {
            "state_bump": self.state_bump,
            "lease_bumps": self.lease_bumps,
            "permission_bumps": self.permission_bumps,
            "nonce": self.nonce,
            "fail_open_on_account_mismatch": self.fail_open_on_account_mismatch,
        }

    def to_json(self) -> CrankPopParamsJSON:
        return {
            "state_bump": self.state_bump,
            "lease_bumps": list(self.lease_bumps),
            "permission_bumps": list(self.permission_bumps),
            "nonce": self.nonce,
            "fail_open_on_account_mismatch": self.fail_open_on_account_mismatch,
        }

    @classmethod
    def from_json(cls, obj: CrankPopParamsJSON) -> "CrankPopParams":
        return cls(
            state_bump=obj["state_bump"],
            lease_bumps=bytes(obj["lease_bumps"]),
            permission_bumps=bytes(obj["permission_bumps"]),
            nonce=obj["nonce"],
            fail_open_on_account_mismatch=obj["fail_open_on_account_mismatch"],
        )
