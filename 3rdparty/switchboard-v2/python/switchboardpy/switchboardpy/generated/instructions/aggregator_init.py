from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorInitArgs(typing.TypedDict):
    params: types.aggregator_init_params.AggregatorInitParams


layout = borsh.CStruct(
    "params" / types.aggregator_init_params.AggregatorInitParams.layout
)


class AggregatorInitAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    queue: PublicKey
    author_wallet: PublicKey
    program_state: PublicKey


def aggregator_init(
    args: AggregatorInitArgs, accounts: AggregatorInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["author_wallet"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xc8)X\x0b$\x15\xb5n"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
