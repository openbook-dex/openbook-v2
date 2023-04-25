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


class LeaseAccountDataJSON(typing.TypedDict):
    escrow: str
    queue: str
    aggregator: str
    token_program: str
    is_active: bool
    crank_row_count: int
    created_at: int
    update_count: int
    withdraw_authority: str
    ebuf: list[int]


@dataclass
class LeaseAccountData:
    discriminator: typing.ClassVar = b"7\xfe\xd0\xfb\xa4,\x962"
    layout: typing.ClassVar = borsh.CStruct(
        "escrow" / BorshPubkey,
        "queue" / BorshPubkey,
        "aggregator" / BorshPubkey,
        "token_program" / BorshPubkey,
        "is_active" / borsh.Bool,
        "crank_row_count" / borsh.U32,
        "created_at" / borsh.I64,
        "update_count" / borsh.U128,
        "withdraw_authority" / BorshPubkey,
        "ebuf" / borsh.U8[256],
    )
    escrow: PublicKey
    queue: PublicKey
    aggregator: PublicKey
    token_program: PublicKey
    is_active: bool
    crank_row_count: int
    created_at: int
    update_count: int
    withdraw_authority: PublicKey
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["LeaseAccountData"]:
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
    ) -> typing.List[typing.Optional["LeaseAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["LeaseAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "LeaseAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = LeaseAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            escrow=dec.escrow,
            queue=dec.queue,
            aggregator=dec.aggregator,
            token_program=dec.token_program,
            is_active=dec.is_active,
            crank_row_count=dec.crank_row_count,
            created_at=dec.created_at,
            update_count=dec.update_count,
            withdraw_authority=dec.withdraw_authority,
            ebuf=dec.ebuf,
        )

    def to_json(self) -> LeaseAccountDataJSON:
        return {
            "escrow": str(self.escrow),
            "queue": str(self.queue),
            "aggregator": str(self.aggregator),
            "token_program": str(self.token_program),
            "is_active": self.is_active,
            "crank_row_count": self.crank_row_count,
            "created_at": self.created_at,
            "update_count": self.update_count,
            "withdraw_authority": str(self.withdraw_authority),
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: LeaseAccountDataJSON) -> "LeaseAccountData":
        return cls(
            escrow=PublicKey(obj["escrow"]),
            queue=PublicKey(obj["queue"]),
            aggregator=PublicKey(obj["aggregator"]),
            token_program=PublicKey(obj["token_program"]),
            is_active=obj["is_active"],
            crank_row_count=obj["crank_row_count"],
            created_at=obj["created_at"],
            update_count=obj["update_count"],
            withdraw_authority=PublicKey(obj["withdraw_authority"]),
            ebuf=obj["ebuf"],
        )
