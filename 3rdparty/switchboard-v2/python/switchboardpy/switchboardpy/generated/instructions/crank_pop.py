from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class CrankPopArgs(typing.TypedDict):
    params: types.crank_pop_params.CrankPopParams


layout = borsh.CStruct("params" / types.crank_pop_params.CrankPopParams.layout)


class CrankPopAccounts(typing.TypedDict):
    crank: PublicKey
    oracle_queue: PublicKey
    queue_authority: PublicKey
    program_state: PublicKey
    payout_wallet: PublicKey
    token_program: PublicKey
    crank_data_buffer: PublicKey
    queue_data_buffer: PublicKey


def crank_pop(args: CrankPopArgs, accounts: CrankPopAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["crank"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["queue_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["payout_wallet"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["crank_data_buffer"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["queue_data_buffer"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"B9\xd8\xfb\xa5k\x80b"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
