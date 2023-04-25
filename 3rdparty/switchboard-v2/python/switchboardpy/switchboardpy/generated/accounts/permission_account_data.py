import typing
from dataclasses import dataclass
from base64 import b64decode
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Commitment
import borsh_construct as borsh
from anchorpy.coder.accounts import ACCOUNT_DISCRIMINATOR_SIZE
from anchorpy.error import AccountInvalidDiscriminator
from anchorpy.utils.rpc import get_multiple_accounts
from anchorpy.borsh_extension import BorshPubkey
from ..program_id import PROGRAM_ID


class PermissionAccountDataJSON(typing.TypedDict):
    authority: str
    permissions: int
    granter: str
    grantee: str
    expiration: int
    ebuf: list[int]


@dataclass
class PermissionAccountData:
    discriminator: typing.ClassVar = b"M%\xb1\xa4&'\"m"
    layout: typing.ClassVar = borsh.CStruct(
        "authority" / BorshPubkey,
        "permissions" / borsh.U32,
        "granter" / BorshPubkey,
        "grantee" / BorshPubkey,
        "expiration" / borsh.I64,
        "ebuf" / borsh.U8[256],
    )
    authority: PublicKey
    permissions: int
    granter: PublicKey
    grantee: PublicKey
    expiration: int
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["PermissionAccountData"]:
        resp = await conn.get_account_info(address, commitment=commitment)
        info = resp["result"]["value"]
        if info is None:
            return None
        if info["owner"] != str(PROGRAM_ID):
            raise ValueError("Account does not belong to this program")
        bytes_data = b64decode(info["data"][0])
        return cls.decode(bytes_data)

    @classmethod
    async def fetch_multiple(
        cls,
        conn: AsyncClient,
        addresses: list[PublicKey],
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.List[typing.Optional["PermissionAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["PermissionAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "PermissionAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = PermissionAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            authority=dec.authority,
            permissions=dec.permissions,
            granter=dec.granter,
            grantee=dec.grantee,
            expiration=dec.expiration,
            ebuf=dec.ebuf,
        )

    def to_json(self) -> PermissionAccountDataJSON:
        return {
            "authority": str(self.authority),
            "permissions": self.permissions,
            "granter": str(self.granter),
            "grantee": str(self.grantee),
            "expiration": self.expiration,
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: PermissionAccountDataJSON) -> "PermissionAccountData":
        return cls(
            authority=PublicKey(obj["authority"]),
            permissions=obj["permissions"],
            granter=PublicKey(obj["granter"]),
            grantee=PublicKey(obj["grantee"]),
            expiration=obj["expiration"],
            ebuf=obj["ebuf"],
        )
