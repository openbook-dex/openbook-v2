from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class ProgramInitArgs(typing.TypedDict):
    params: types.program_init_params.ProgramInitParams


layout = borsh.CStruct("params" / types.program_init_params.ProgramInitParams.layout)


class ProgramInitAccounts(typing.TypedDict):
    state: PublicKey
    authority: PublicKey
    token_mint: PublicKey
    vault: PublicKey
    payer: PublicKey
    system_program: PublicKey
    token_program: PublicKey


def program_init(
    args: ProgramInitArgs, accounts: ProgramInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["state"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["token_mint"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["vault"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["payer"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xc7\xd1\xc1\xd5\x8a\x1e\xaf\r"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
