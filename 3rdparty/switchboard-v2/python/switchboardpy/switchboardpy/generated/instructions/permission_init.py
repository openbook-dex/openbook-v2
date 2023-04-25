from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class PermissionInitArgs(typing.TypedDict):
    params: types.permission_init_params.PermissionInitParams


layout = borsh.CStruct(
    "params" / types.permission_init_params.PermissionInitParams.layout
)


class PermissionInitAccounts(typing.TypedDict):
    permission: PublicKey
    authority: PublicKey
    granter: PublicKey
    grantee: PublicKey
    payer: PublicKey
    system_program: PublicKey


def permission_init(
    args: PermissionInitArgs, accounts: PermissionInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["granter"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["grantee"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["payer"], is_signer=True, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xb1t\xc9\xe9\x10\x02\x0b\xb3"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
