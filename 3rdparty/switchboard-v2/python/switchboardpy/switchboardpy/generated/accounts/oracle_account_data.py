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
from .. import types


class OracleAccountDataJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    oracle_authority: str
    last_heartbeat: int
    num_in_use: int
    token_account: str
    queue_pubkey: str
    metrics: types.oracle_metrics.OracleMetricsJSON
    ebuf: list[int]


@dataclass
class OracleAccountData:
    discriminator: typing.ClassVar = b"\x80\x1e\x10\xf1\xaaI76"
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[128],
        "oracle_authority" / BorshPubkey,
        "last_heartbeat" / borsh.I64,
        "num_in_use" / borsh.U32,
        "token_account" / BorshPubkey,
        "queue_pubkey" / BorshPubkey,
        "metrics" / types.oracle_metrics.OracleMetrics.layout,
        "ebuf" / borsh.U8[256],
    )
    name: list[int]
    metadata: list[int]
    oracle_authority: PublicKey
    last_heartbeat: int
    num_in_use: int
    token_account: PublicKey
    queue_pubkey: PublicKey
    metrics: types.oracle_metrics.OracleMetrics
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["OracleAccountData"]:
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
    ) -> typing.List[typing.Optional["OracleAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["OracleAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "OracleAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = OracleAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            name=dec.name,
            metadata=dec.metadata,
            oracle_authority=dec.oracle_authority,
            last_heartbeat=dec.last_heartbeat,
            num_in_use=dec.num_in_use,
            token_account=dec.token_account,
            queue_pubkey=dec.queue_pubkey,
            metrics=types.oracle_metrics.OracleMetrics.from_decoded(dec.metrics),
            ebuf=dec.ebuf,
        )

    def to_json(self) -> OracleAccountDataJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "oracle_authority": str(self.oracle_authority),
            "last_heartbeat": self.last_heartbeat,
            "num_in_use": self.num_in_use,
            "token_account": str(self.token_account),
            "queue_pubkey": str(self.queue_pubkey),
            "metrics": self.metrics.to_json(),
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: OracleAccountDataJSON) -> "OracleAccountData":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            oracle_authority=PublicKey(obj["oracle_authority"]),
            last_heartbeat=obj["last_heartbeat"],
            num_in_use=obj["num_in_use"],
            token_account=PublicKey(obj["token_account"]),
            queue_pubkey=PublicKey(obj["queue_pubkey"]),
            metrics=types.oracle_metrics.OracleMetrics.from_json(obj["metrics"]),
            ebuf=obj["ebuf"],
        )
