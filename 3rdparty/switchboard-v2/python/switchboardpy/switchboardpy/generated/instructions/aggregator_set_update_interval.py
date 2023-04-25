from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorSetUpdateIntervalArgs(typing.TypedDict):
    params: types.aggregator_set_update_interval_params.AggregatorSetUpdateIntervalParams


layout = borsh.CStruct(
    "params"
    / types.aggregator_set_update_interval_params.AggregatorSetUpdateIntervalParams.layout
)


class AggregatorSetUpdateIntervalAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey


def aggregator_set_update_interval(
    args: AggregatorSetUpdateIntervalArgs, accounts: AggregatorSetUpdateIntervalAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\xb3\x0c\r\x90\xdbXQh"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
