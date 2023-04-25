from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorSetMinOraclesArgs(typing.TypedDict):
    params: types.aggregator_set_min_oracles_params.AggregatorSetMinOraclesParams


layout = borsh.CStruct(
    "params"
    / types.aggregator_set_min_oracles_params.AggregatorSetMinOraclesParams.layout
)


class AggregatorSetMinOraclesAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey


def aggregator_set_min_oracles(
    args: AggregatorSetMinOraclesArgs, accounts: AggregatorSetMinOraclesAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\xb2#GA\x99\xc1\x91\x1c"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
