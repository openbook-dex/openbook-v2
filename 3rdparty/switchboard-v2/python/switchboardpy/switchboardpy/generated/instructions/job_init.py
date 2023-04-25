from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class JobInitArgs(typing.TypedDict):
    params: types.job_init_params.JobInitParams


layout = borsh.CStruct("params" / types.job_init_params.JobInitParams.layout)


class JobInitAccounts(typing.TypedDict):
    job: PublicKey
    author_wallet: PublicKey
    program_state: PublicKey


def job_init(args: JobInitArgs, accounts: JobInitAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["job"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["author_wallet"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b'eVi\xc0"\xc9\x93\x9f'
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
