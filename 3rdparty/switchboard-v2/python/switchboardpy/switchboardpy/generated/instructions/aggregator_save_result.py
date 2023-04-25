from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class AggregatorSaveResultArgs(typing.TypedDict):
    params: types.aggregator_save_result_params.AggregatorSaveResultParams


layout = borsh.CStruct(
    "params" / types.aggregator_save_result_params.AggregatorSaveResultParams.layout
)


class AggregatorSaveResultAccounts(typing.TypedDict):
    aggregator: PublicKey
    oracle: PublicKey
    oracle_authority: PublicKey
    oracle_queue: PublicKey
    queue_authority: PublicKey
    feed_permission: PublicKey
    oracle_permission: PublicKey
    lease: PublicKey
    escrow: PublicKey
    token_program: PublicKey
    program_state: PublicKey
    history_buffer: PublicKey


def aggregator_save_result(
    args: AggregatorSaveResultArgs, accounts: AggregatorSaveResultAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["oracle_authority"], is_signer=True, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["oracle_queue"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["queue_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["feed_permission"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["oracle_permission"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["history_buffer"], is_signer=False, is_writable=True
        ),
    ]
    identifier = b"\x15C\x05\x00J\xa83\xc0"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
