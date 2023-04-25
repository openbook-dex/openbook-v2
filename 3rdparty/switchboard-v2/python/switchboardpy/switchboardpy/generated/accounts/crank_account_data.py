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


class CrankAccountDataJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    queue_pubkey: str
    pq_size: int
    max_rows: int
    jitter_modifier: int
    ebuf: list[int]
    data_buffer: str


@dataclass
class CrankAccountData:
    discriminator: typing.ClassVar = b"oQ\x92I\xac\xb4\x86\xd1"
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[64],
        "queue_pubkey" / BorshPubkey,
        "pq_size" / borsh.U32,
        "max_rows" / borsh.U32,
        "jitter_modifier" / borsh.U8,
        "ebuf" / borsh.U8[255],
        "data_buffer" / BorshPubkey,
    )
    name: list[int]
    metadata: list[int]
    queue_pubkey: PublicKey
    pq_size: int
    max_rows: int
    jitter_modifier: int
    ebuf: list[int]
    data_buffer: PublicKey

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["CrankAccountData"]:
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
    ) -> typing.List[typing.Optional["CrankAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["CrankAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "CrankAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = CrankAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            name=dec.name,
            metadata=dec.metadata,
            queue_pubkey=dec.queue_pubkey,
            pq_size=dec.pq_size,
            max_rows=dec.max_rows,
            jitter_modifier=dec.jitter_modifier,
            ebuf=dec.ebuf,
            data_buffer=dec.data_buffer,
        )

    def to_json(self) -> CrankAccountDataJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "queue_pubkey": str(self.queue_pubkey),
            "pq_size": self.pq_size,
            "max_rows": self.max_rows,
            "jitter_modifier": self.jitter_modifier,
            "ebuf": self.ebuf,
            "data_buffer": str(self.data_buffer),
        }

    @classmethod
    def from_json(cls, obj: CrankAccountDataJSON) -> "CrankAccountData":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            queue_pubkey=PublicKey(obj["queue_pubkey"]),
            pq_size=obj["pq_size"],
            max_rows=obj["max_rows"],
            jitter_modifier=obj["jitter_modifier"],
            ebuf=obj["ebuf"],
            data_buffer=PublicKey(obj["data_buffer"]),
        )
