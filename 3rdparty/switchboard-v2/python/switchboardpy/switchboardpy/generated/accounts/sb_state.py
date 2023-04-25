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


class SbStateJSON(typing.TypedDict):
    authority: str
    token_mint: str
    token_vault: str
    ebuf: list[int]


@dataclass
class SbState:
    discriminator: typing.ClassVar = b"\x9f*\xc0\xbf\x8b>\xa8\x1c"
    layout: typing.ClassVar = borsh.CStruct(
        "authority" / BorshPubkey,
        "token_mint" / BorshPubkey,
        "token_vault" / BorshPubkey,
        "ebuf" / borsh.U8[1024],
    )
    authority: PublicKey
    token_mint: PublicKey
    token_vault: PublicKey
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["SbState"]:
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
    ) -> typing.List[typing.Optional["SbState"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["SbState"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "SbState":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = SbState.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            authority=dec.authority,
            token_mint=dec.token_mint,
            token_vault=dec.token_vault,
            ebuf=dec.ebuf,
        )

    def to_json(self) -> SbStateJSON:
        return {
            "authority": str(self.authority),
            "token_mint": str(self.token_mint),
            "token_vault": str(self.token_vault),
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: SbStateJSON) -> "SbState":
        return cls(
            authority=PublicKey(obj["authority"]),
            token_mint=PublicKey(obj["token_mint"]),
            token_vault=PublicKey(obj["token_vault"]),
            ebuf=obj["ebuf"],
        )
