from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class PermissionSetArgs(typing.TypedDict):
    params: types.permission_set_params.PermissionSetParams


layout = borsh.CStruct(
    "params" / types.permission_set_params.PermissionSetParams.layout
)


class PermissionSetAccounts(typing.TypedDict):
    permission: PublicKey
    authority: PublicKey


def permission_set(
    args: PermissionSetArgs, accounts: PermissionSetAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"\xd3z\xb9x\x81\xb67g"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
