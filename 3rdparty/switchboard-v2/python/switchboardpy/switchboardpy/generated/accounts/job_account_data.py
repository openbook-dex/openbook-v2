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


class JobAccountDataJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    author_wallet: str
    expiration: int
    hash: list[int]
    data: list[int]
    reference_count: int
    total_spent: int


@dataclass
class JobAccountData:
    discriminator: typing.ClassVar = b"|Ee\xc3\xe5\xda\x90?"
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[64],
        "author_wallet" / BorshPubkey,
        "expiration" / borsh.I64,
        "hash" / borsh.U8[32],
        "data" / borsh.Bytes,
        "reference_count" / borsh.U32,
        "total_spent" / borsh.U128,
    )
    name: list[int]
    metadata: list[int]
    author_wallet: PublicKey
    expiration: int
    hash: list[int]
    data: bytes
    reference_count: int
    total_spent: int

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["JobAccountData"]:
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
    ) -> typing.List[typing.Optional["JobAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["JobAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "JobAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = JobAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            name=dec.name,
            metadata=dec.metadata,
            author_wallet=dec.author_wallet,
            expiration=dec.expiration,
            hash=dec.hash,
            data=dec.data,
            reference_count=dec.reference_count,
            total_spent=dec.total_spent,
        )

    def to_json(self) -> JobAccountDataJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "author_wallet": str(self.author_wallet),
            "expiration": self.expiration,
            "hash": self.hash,
            "data": list(self.data),
            "reference_count": self.reference_count,
            "total_spent": self.total_spent,
        }

    @classmethod
    def from_json(cls, obj: JobAccountDataJSON) -> "JobAccountData":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            author_wallet=PublicKey(obj["author_wallet"]),
            expiration=obj["expiration"],
            hash=obj["hash"],
            data=bytes(obj["data"]),
            reference_count=obj["reference_count"],
            total_spent=obj["total_spent"],
        )
