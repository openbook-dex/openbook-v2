from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class CrankInitArgs(typing.TypedDict):
    params: types.crank_init_params.CrankInitParams


layout = borsh.CStruct("params" / types.crank_init_params.CrankInitParams.layout)


class CrankInitAccounts(typing.TypedDict):
    crank: PublicKey
    queue: PublicKey
    buffer: PublicKey
    payer: PublicKey
    system_program: PublicKey


def crank_init(
    args: CrankInitArgs, accounts: CrankInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["crank"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["buffer"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["payer"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"9\xb3^\x88RO\x19\xb9"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
