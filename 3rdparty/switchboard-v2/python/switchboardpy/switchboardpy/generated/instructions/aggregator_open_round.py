from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorOpenRoundArgs(typing.TypedDict):
    params: types.aggregator_open_round_params.AggregatorOpenRoundParams


layout = borsh.CStruct(
    "params" / types.aggregator_open_round_params.AggregatorOpenRoundParams.layout
)


class AggregatorOpenRoundAccounts(typing.TypedDict):
    aggregator: PublicKey
    lease: PublicKey
    oracle_queue: PublicKey
    queue_authority: PublicKey
    permission: PublicKey
    escrow: PublicKey
    program_state: PublicKey
    payout_wallet: PublicKey
    token_program: PublicKey
    data_buffer: PublicKey


def aggregator_open_round(
    args: AggregatorOpenRoundArgs, accounts: AggregatorOpenRoundAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["queue_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["payout_wallet"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["data_buffer"], is_signer=False, is_writable=False),
    ]
    identifier = b"\xefE\xe5\xb3\x9c\xf6v\xbf"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
