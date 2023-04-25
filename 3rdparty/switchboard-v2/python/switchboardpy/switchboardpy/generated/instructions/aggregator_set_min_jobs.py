from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorSetMinJobsArgs(typing.TypedDict):
    params: types.aggregator_set_min_jobs_params.AggregatorSetMinJobsParams


layout = borsh.CStruct(
    "params" / types.aggregator_set_min_jobs_params.AggregatorSetMinJobsParams.layout
)


class AggregatorSetMinJobsAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey


def aggregator_set_min_jobs(
    args: AggregatorSetMinJobsArgs, accounts: AggregatorSetMinJobsAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\x9eM\x95\x9c\x9d*\x19\x10"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
