from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleQueueSetRewardsArgs(typing.TypedDict):
    params: types.oracle_queue_set_rewards_params.OracleQueueSetRewardsParams


layout = borsh.CStruct(
    "params" / types.oracle_queue_set_rewards_params.OracleQueueSetRewardsParams.layout
)


class OracleQueueSetRewardsAccounts(typing.TypedDict):
    queue: PublicKey
    authority: PublicKey


def oracle_queue_set_rewards(
    args: OracleQueueSetRewardsArgs, accounts: OracleQueueSetRewardsAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\xab\xc8\xe9\x17\x83\xa0\xe3u"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
