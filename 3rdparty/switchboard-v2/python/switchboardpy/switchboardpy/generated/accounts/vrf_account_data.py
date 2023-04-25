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


class VrfAccountDataJSON(typing.TypedDict):
    status: types.vrf_status.VrfStatusJSON
    counter: int
    authority: str
    oracle_queue: str
    escrow: str
    callback: types.callback_zc.CallbackZCJSON
    batch_size: int
    builders: list[types.vrf_builder.VrfBuilderJSON]
    builders_len: int
    test_mode: bool
    current_round: types.vrf_round.VrfRoundJSON
    ebuf: list[int]


@dataclass
class VrfAccountData:
    discriminator: typing.ClassVar = b"e#>\xefg\x97\x06\x12"
    layout: typing.ClassVar = borsh.CStruct(
        "status" / types.vrf_status.layout,
        "counter" / borsh.U128,
        "authority" / BorshPubkey,
        "oracle_queue" / BorshPubkey,
        "escrow" / BorshPubkey,
        "callback" / types.callback_zc.CallbackZC.layout,
        "batch_size" / borsh.U32,
        "builders" / types.vrf_builder.VrfBuilder.layout[8],
        "builders_len" / borsh.U32,
        "test_mode" / borsh.Bool,
        "current_round" / types.vrf_round.VrfRound.layout,
        "ebuf" / borsh.U8[1024],
    )
    status: types.vrf_status.VrfStatusKind
    counter: int
    authority: PublicKey
    oracle_queue: PublicKey
    escrow: PublicKey
    callback: types.callback_zc.CallbackZC
    batch_size: int
    builders: list[types.vrf_builder.VrfBuilder]
    builders_len: int
    test_mode: bool
    current_round: types.vrf_round.VrfRound
    ebuf: list[int]

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: PublicKey,
        commitment: typing.Optional[Commitment] = None,
    ) -> typing.Optional["VrfAccountData"]:
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
    ) -> typing.List[typing.Optional["VrfAccountData"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["VrfAccountData"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != PROGRAM_ID:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "VrfAccountData":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = VrfAccountData.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            status=types.vrf_status.from_decoded(dec.status),
            counter=dec.counter,
            authority=dec.authority,
            oracle_queue=dec.oracle_queue,
            escrow=dec.escrow,
            callback=types.callback_zc.CallbackZC.from_decoded(dec.callback),
            batch_size=dec.batch_size,
            builders=list(
                map(
                    lambda item: types.vrf_builder.VrfBuilder.from_decoded(item),
                    dec.builders,
                )
            ),
            builders_len=dec.builders_len,
            test_mode=dec.test_mode,
            current_round=types.vrf_round.VrfRound.from_decoded(dec.current_round),
            ebuf=dec.ebuf,
        )

    def to_json(self) -> VrfAccountDataJSON:
        return {
            "status": self.status.to_json(),
            "counter": self.counter,
            "authority": str(self.authority),
            "oracle_queue": str(self.oracle_queue),
            "escrow": str(self.escrow),
            "callback": self.callback.to_json(),
            "batch_size": self.batch_size,
            "builders": list(map(lambda item: item.to_json(), self.builders)),
            "builders_len": self.builders_len,
            "test_mode": self.test_mode,
            "current_round": self.current_round.to_json(),
            "ebuf": self.ebuf,
        }

    @classmethod
    def from_json(cls, obj: VrfAccountDataJSON) -> "VrfAccountData":
        return cls(
            status=types.vrf_status.from_json(obj["status"]),
            counter=obj["counter"],
            authority=PublicKey(obj["authority"]),
            oracle_queue=PublicKey(obj["oracle_queue"]),
            escrow=PublicKey(obj["escrow"]),
            callback=types.callback_zc.CallbackZC.from_json(obj["callback"]),
            batch_size=obj["batch_size"],
            builders=list(
                map(
                    lambda item: types.vrf_builder.VrfBuilder.from_json(item),
                    obj["builders"],
                )
            ),
            builders_len=obj["builders_len"],
            test_mode=obj["test_mode"],
            current_round=types.vrf_round.VrfRound.from_json(obj["current_round"]),
            ebuf=obj["ebuf"],
        )
