from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleQueueInitArgs(typing.TypedDict):
    params: types.oracle_queue_init_params.OracleQueueInitParams


layout = borsh.CStruct(
    "params" / types.oracle_queue_init_params.OracleQueueInitParams.layout
)


class OracleQueueInitAccounts(typing.TypedDict):
    oracle_queue: PublicKey
    authority: PublicKey
    buffer: PublicKey
    payer: PublicKey
    system_program: PublicKey


def oracle_queue_init(
    args: OracleQueueInitArgs, accounts: OracleQueueInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["buffer"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["payer"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xfa\xe2\xe7o\x9e\xa4\x1b\x88"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
