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


class AggregatorAccountDataJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    author_wallet: str
    queue_pubkey: str
    oracle_request_batch_size: int
    min_oracle_results: int
    min_job_results: int
    min_update_delay_seconds: int
    start_after: int
    variance_threshold: types.switchboard_decimal.SwitchboardDecimalJSON
    force_report_period: int
    expiration: int
    consecutive_failure_count: int
    next_allowed_update_time: int
    is_locked: bool
    crank_pubkey: str
    latest_confirmed_round: types.aggregator_round.AggregatorRoundJSON
    current_round: types.aggregator_round.AggregatorRoundJSON
    job_pubkeys_data: list[str]
    job_hashes: list[types.hash.HashJSON]
    job_pubkeys_size: int
    jobs_checksum: list[int]
    authority: str
    history_buffer: str
    previous_confirmed_round_result: types.switchboard_decimal.SwitchboardDecimalJSON
    previous_confirmed_round_slot: int
    disable_crank: bool
    ebuf: list[int]


@dataclass
class AggregatorAccountData:
    discriminator: typing.ClassVar = b"\xd9\xe6Ae\xc9\xa2\x1b}"
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[128],
        "author_wallet" / BorshPubkey,
        "queue_pubkey" / BorshPubkey,
        "oracle_request_batch_size" / borsh.U32,
        "min_oracle_results" / borsh.U32,
        "min_job_results" / borsh.U32,
        "min_update_delay_seconds" / borsh.U32,
        "start_after" / borsh.I64,
        "variance_threshold" / types.switchboard_decimal.SwitchboardDecimal.layout,
        "force_report_period" / borsh.I64,
        "expiration" / borsh.I64,
        "consecutive_failure_count" / borsh.U64,
        "next_allowed_update_time" / borsh.I64,
        "is_locked" / borsh.Bool,
        "crank_pubkey" / BorshPubkey,
        "latest_confirmed_round" / types.aggregator_round.AggregatorRound.layout,
        "current_round" / types.aggregator_round.AggregatorRound.layout,
        "job_pubkeys_data" / BorshPubkey[16],
        "job_hashes" / types.hash.Hash.layout[16],
        "job_pubkeys_size" / borsh.U32,
        "jobs_checksum" / borsh.U8[32],
        "authority" / BorshPubkey,
        "history_buffer" / BorshPubkey,
        "previous_confirmed_round_result"
        / types.switchboard_decimal.SwitchboardDecimal.layout,
        "previous_confirmed_round_slot" / borsh.U64,
        "disable_crank" / borsh.Bool,
        "ebuf" / borsh.U8[163],
    )
    name: list[int]
    metadata: list[int]
    author_wallet: PublicKey
    queue_pubkey: PublicKey
    oracle_request_batch_size: int
    min_oracle_results: int
    min_job_results: int
    min_update_delay_seconds: int
    start_after: int
    variance_threshold: types.switchboard_decimal.SwitchboardDecimal
    force_report_period: int
    expiration: int
    consecutive_failure_count: int
    next_allowed_update_time: int
    is_locked: bool
    crank_pubkey: PublicKey
    latest_confirmed_round: types.aggregator_round.AggregatorRound
    current_round: types.aggregator_round.AggregatorRound
    job_pubkeys_data: list[PublicKey]
    job_hashes: list[types.hash.Hash]
    job_pubkeys_size: int
    jobs_checksum: list[int]
    authority: PublicKey
    history_buffer: PublicKey
    previous_confirmed_round_result: types.switchboard_decimal.SwitchboardDecimal
    previous_confirmed_round_slot: int
    disable_crank: bool
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["AggregatorAccountData"]:
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
    ) -> typing.List[typing.Optional["AggregatorAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["AggregatorAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "AggregatorAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = AggregatorAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            name=dec.name,
            metadata=dec.metadata,
            author_wallet=dec.author_wallet,
            queue_pubkey=dec.queue_pubkey,
            oracle_request_batch_size=dec.oracle_request_batch_size,
            min_oracle_results=dec.min_oracle_results,
            min_job_results=dec.min_job_results,
            min_update_delay_seconds=dec.min_update_delay_seconds,
            start_after=dec.start_after,
            variance_threshold=types.switchboard_decimal.SwitchboardDecimal.from_decoded(
                dec.variance_threshold
            ),
            force_report_period=dec.force_report_period,
            expiration=dec.expiration,
            consecutive_failure_count=dec.consecutive_failure_count,
            next_allowed_update_time=dec.next_allowed_update_time,
            is_locked=dec.is_locked,
            crank_pubkey=dec.crank_pubkey,
            latest_confirmed_round=types.aggregator_round.AggregatorRound.from_decoded(
                dec.latest_confirmed_round
            ),
            current_round=types.aggregator_round.AggregatorRound.from_decoded(
                dec.current_round
            ),
            job_pubkeys_data=dec.job_pubkeys_data,
            job_hashes=list(
                map(lambda item: types.hash.Hash.from_decoded(item), dec.job_hashes)
            ),
            job_pubkeys_size=dec.job_pubkeys_size,
            jobs_checksum=dec.jobs_checksum,
            authority=dec.authority,
            history_buffer=dec.history_buffer,
            previous_confirmed_round_result=types.switchboard_decimal.SwitchboardDecimal.from_decoded(
                dec.previous_confirmed_round_result
            ),
            previous_confirmed_round_slot=dec.previous_confirmed_round_slot,
            disable_crank=dec.disable_crank,
            ebuf=dec.ebuf,
        )

    def to_json(self) -> AggregatorAccountDataJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "author_wallet": str(self.author_wallet),
            "queue_pubkey": str(self.queue_pubkey),
            "oracle_request_batch_size": self.oracle_request_batch_size,
            "min_oracle_results": self.min_oracle_results,
            "min_job_results": self.min_job_results,
            "min_update_delay_seconds": self.min_update_delay_seconds,
            "start_after": self.start_after,
            "variance_threshold": self.variance_threshold.to_json(),
            "force_report_period": self.force_report_period,
            "expiration": self.expiration,
            "consecutive_failure_count": self.consecutive_failure_count,
            "next_allowed_update_time": self.next_allowed_update_time,
            "is_locked": self.is_locked,
            "crank_pubkey": str(self.crank_pubkey),
            "latest_confirmed_round": self.latest_confirmed_round.to_json(),
            "current_round": self.current_round.to_json(),
            "job_pubkeys_data": list(
                map(lambda item: str(item), self.job_pubkeys_data)
            ),
            "job_hashes": list(map(lambda item: item.to_json(), self.job_hashes)),
            "job_pubkeys_size": self.job_pubkeys_size,
            "jobs_checksum": self.jobs_checksum,
            "authority": str(self.authority),
            "history_buffer": str(self.history_buffer),
            "previous_confirmed_round_result": self.previous_confirmed_round_result.to_json(),
            "previous_confirmed_round_slot": self.previous_confirmed_round_slot,
            "disable_crank": self.disable_crank,
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: AggregatorAccountDataJSON) -> "AggregatorAccountData":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            author_wallet=PublicKey(obj["author_wallet"]),
            queue_pubkey=PublicKey(obj["queue_pubkey"]),
            oracle_request_batch_size=obj["oracle_request_batch_size"],
            min_oracle_results=obj["min_oracle_results"],
            min_job_results=obj["min_job_results"],
            min_update_delay_seconds=obj["min_update_delay_seconds"],
            start_after=obj["start_after"],
            variance_threshold=types.switchboard_decimal.SwitchboardDecimal.from_json(
                obj["variance_threshold"]
            ),
            force_report_period=obj["force_report_period"],
            expiration=obj["expiration"],
            consecutive_failure_count=obj["consecutive_failure_count"],
            next_allowed_update_time=obj["next_allowed_update_time"],
            is_locked=obj["is_locked"],
            crank_pubkey=PublicKey(obj["crank_pubkey"]),
            latest_confirmed_round=types.aggregator_round.AggregatorRound.from_json(
                obj["latest_confirmed_round"]
            ),
            current_round=types.aggregator_round.AggregatorRound.from_json(
                obj["current_round"]
            ),
            job_pubkeys_data=list(
                map(lambda item: PublicKey(item), obj["job_pubkeys_data"])
            ),
            job_hashes=list(
                map(lambda item: types.hash.Hash.from_json(item), obj["job_hashes"])
            ),
            job_pubkeys_size=obj["job_pubkeys_size"],
            jobs_checksum=obj["jobs_checksum"],
            authority=PublicKey(obj["authority"]),
            history_buffer=PublicKey(obj["history_buffer"]),
            previous_confirmed_round_result=types.switchboard_decimal.SwitchboardDecimal.from_json(
                obj["previous_confirmed_round_result"]
            ),
            previous_confirmed_round_slot=obj["previous_confirmed_round_slot"],
            disable_crank=obj["disable_crank"],
            ebuf=obj["ebuf"],
        )
