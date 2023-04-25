from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorRemoveJobArgs(typing.TypedDict):
    params: types.aggregator_remove_job_params.AggregatorRemoveJobParams


layout = borsh.CStruct(
    "params" / types.aggregator_remove_job_params.AggregatorRemoveJobParams.layout
)


class AggregatorRemoveJobAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    job: PublicKey


def aggregator_remove_job(
    args: AggregatorRemoveJobArgs, accounts: AggregatorRemoveJobAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["job"], is_signer=False, is_writable=True),
    ]
    identifier = b"\x9e\xdd\xe7A)\x97\x9b\xac"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
