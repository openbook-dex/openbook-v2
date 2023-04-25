from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class CrankPushArgs(typing.TypedDict):
    params: types.crank_push_params.CrankPushParams


layout = borsh.CStruct("params" / types.crank_push_params.CrankPushParams.layout)


class CrankPushAccounts(typing.TypedDict):
    crank: PublicKey
    aggregator: PublicKey
    oracle_queue: PublicKey
    queue_authority: PublicKey
    permission: PublicKey
    lease: PublicKey
    escrow: PublicKey
    program_state: PublicKey
    data_buffer: PublicKey


def crank_push(
    args: CrankPushArgs, accounts: CrankPushAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["crank"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["queue_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["data_buffer"], is_signer=False, is_writable=True),
    ]
    identifier = b"\x9b\xaf\xa0\x12\x07\x93\xf9\x10"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
