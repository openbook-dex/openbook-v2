from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorSetBatchSizeArgs(typing.TypedDict):
    params: types.aggregator_set_batch_size_params.AggregatorSetBatchSizeParams


layout = borsh.CStruct(
    "params"
    / types.aggregator_set_batch_size_params.AggregatorSetBatchSizeParams.layout
)


class AggregatorSetBatchSizeAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey


def aggregator_set_batch_size(
    args: AggregatorSetBatchSizeArgs, accounts: AggregatorSetBatchSizeAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\xaaW\xbb\xf7\xb5\x9c\x8fV"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
