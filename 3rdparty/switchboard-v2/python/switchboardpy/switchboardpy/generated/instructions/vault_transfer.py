from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class VaultTransferArgs(typing.TypedDict):
    params: types.vault_transfer_params.VaultTransferParams


layout = borsh.CStruct(
    "params" / types.vault_transfer_params.VaultTransferParams.layout
)


class VaultTransferAccounts(typing.TypedDict):
    state: PublicKey
    authority: PublicKey
    to: PublicKey
    vault: PublicKey
    token_program: PublicKey


def vault_transfer(
    args: VaultTransferArgs, accounts: VaultTransferAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["state"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["to"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["vault"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xd3}\x03i-!\xe3\xd6"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
