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


class OracleQueueAccountDataJSON(typing.TypedDict):
    name: list[int]
    metadata: list[int]
    authority: str
    oracle_timeout: int
    reward: int
    min_stake: int
    slashing_enabled: bool
    variance_tolerance_multiplier: types.switchboard_decimal.SwitchboardDecimalJSON
    feed_probation_period: int
    curr_idx: int
    size: int
    gc_idx: int
    consecutive_feed_failure_limit: int
    consecutive_oracle_failure_limit: int
    unpermissioned_feeds_enabled: bool
    unpermissioned_vrf_enabled: bool
    curator_reward_cut: types.switchboard_decimal.SwitchboardDecimalJSON
    lock_lease_funding: bool
    ebuf: list[int]
    max_size: int
    data_buffer: str


@dataclass
class OracleQueueAccountData:
    discriminator: typing.ClassVar = b"\xa4\xcf\xc83\xc7q#m"
    layout: typing.ClassVar = borsh.CStruct(
        "name" / borsh.U8[32],
        "metadata" / borsh.U8[64],
        "authority" / BorshPubkey,
        "oracle_timeout" / borsh.U32,
        "reward" / borsh.U64,
        "min_stake" / borsh.U64,
        "slashing_enabled" / borsh.Bool,
        "variance_tolerance_multiplier"
        / types.switchboard_decimal.SwitchboardDecimal.layout,
        "feed_probation_period" / borsh.U32,
        "curr_idx" / borsh.U32,
        "size" / borsh.U32,
        "gc_idx" / borsh.U32,
        "consecutive_feed_failure_limit" / borsh.U64,
        "consecutive_oracle_failure_limit" / borsh.U64,
        "unpermissioned_feeds_enabled" / borsh.Bool,
        "unpermissioned_vrf_enabled" / borsh.Bool,
        "curator_reward_cut" / types.switchboard_decimal.SwitchboardDecimal.layout,
        "lock_lease_funding" / borsh.Bool,
        "ebuf" / borsh.U8[1001],
        "max_size" / borsh.U32,
        "data_buffer" / BorshPubkey,
    )
    name: list[int]
    metadata: list[int]
    authority: PublicKey
    oracle_timeout: int
    reward: int
    min_stake: int
    slashing_enabled: bool
    variance_tolerance_multiplier: types.switchboard_decimal.SwitchboardDecimal
    feed_probation_period: int
    curr_idx: int
    size: int
    gc_idx: int
    consecutive_feed_failure_limit: int
    consecutive_oracle_failure_limit: int
    unpermissioned_feeds_enabled: bool
    unpermissioned_vrf_enabled: bool
    curator_reward_cut: types.switchboard_decimal.SwitchboardDecimal
    lock_lease_funding: bool
    ebuf: list[int]
    max_size: int
    data_buffer: PublicKey

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["OracleQueueAccountData"]:
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
    ) -> typing.List[typing.Optional["OracleQueueAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["OracleQueueAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "OracleQueueAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = OracleQueueAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            name=dec.name,
            metadata=dec.metadata,
            authority=dec.authority,
            oracle_timeout=dec.oracle_timeout,
            reward=dec.reward,
            min_stake=dec.min_stake,
            slashing_enabled=dec.slashing_enabled,
            variance_tolerance_multiplier=types.switchboard_decimal.SwitchboardDecimal.from_decoded(
                dec.variance_tolerance_multiplier
            ),
            feed_probation_period=dec.feed_probation_period,
            curr_idx=dec.curr_idx,
            size=dec.size,
            gc_idx=dec.gc_idx,
            consecutive_feed_failure_limit=dec.consecutive_feed_failure_limit,
            consecutive_oracle_failure_limit=dec.consecutive_oracle_failure_limit,
            unpermissioned_feeds_enabled=dec.unpermissioned_feeds_enabled,
            unpermissioned_vrf_enabled=dec.unpermissioned_vrf_enabled,
            curator_reward_cut=types.switchboard_decimal.SwitchboardDecimal.from_decoded(
                dec.curator_reward_cut
            ),
            lock_lease_funding=dec.lock_lease_funding,
            ebuf=dec.ebuf,
            max_size=dec.max_size,
            data_buffer=dec.data_buffer,
        )

    def to_json(self) -> OracleQueueAccountDataJSON:
        return {
            "name": self.name,
            "metadata": self.metadata,
            "authority": str(self.authority),
            "oracle_timeout": self.oracle_timeout,
            "reward": self.reward,
            "min_stake": self.min_stake,
            "slashing_enabled": self.slashing_enabled,
            "variance_tolerance_multiplier": self.variance_tolerance_multiplier.to_json(),
            "feed_probation_period": self.feed_probation_period,
            "curr_idx": self.curr_idx,
            "size": self.size,
            "gc_idx": self.gc_idx,
            "consecutive_feed_failure_limit": self.consecutive_feed_failure_limit,
            "consecutive_oracle_failure_limit": self.consecutive_oracle_failure_limit,
            "unpermissioned_feeds_enabled": self.unpermissioned_feeds_enabled,
            "unpermissioned_vrf_enabled": self.unpermissioned_vrf_enabled,
            "curator_reward_cut": self.curator_reward_cut.to_json(),
            "lock_lease_funding": self.lock_lease_funding,
            "ebuf": self.ebuf,
            "max_size": self.max_size,
            "data_buffer": str(self.data_buffer),
        }

    @classmethod
    def from_json(cls, obj: OracleQueueAccountDataJSON) -> "OracleQueueAccountData":
        return cls(
            name=obj["name"],
            metadata=obj["metadata"],
            authority=PublicKey(obj["authority"]),
            oracle_timeout=obj["oracle_timeout"],
            reward=obj["reward"],
            min_stake=obj["min_stake"],
            slashing_enabled=obj["slashing_enabled"],
            variance_tolerance_multiplier=types.switchboard_decimal.SwitchboardDecimal.from_json(
                obj["variance_tolerance_multiplier"]
            ),
            feed_probation_period=obj["feed_probation_period"],
            curr_idx=obj["curr_idx"],
            size=obj["size"],
            gc_idx=obj["gc_idx"],
            consecutive_feed_failure_limit=obj["consecutive_feed_failure_limit"],
            consecutive_oracle_failure_limit=obj["consecutive_oracle_failure_limit"],
            unpermissioned_feeds_enabled=obj["unpermissioned_feeds_enabled"],
            unpermissioned_vrf_enabled=obj["unpermissioned_vrf_enabled"],
            curator_reward_cut=types.switchboard_decimal.SwitchboardDecimal.from_json(
                obj["curator_reward_cut"]
            ),
            lock_lease_funding=obj["lock_lease_funding"],
            ebuf=obj["ebuf"],
            max_size=obj["max_size"],
            data_buffer=PublicKey(obj["data_buffer"]),
        )
