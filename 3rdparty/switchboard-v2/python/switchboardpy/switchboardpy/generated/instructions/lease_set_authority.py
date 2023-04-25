from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class LeaseSetAuthorityAccounts(typing.TypedDict):
    lease: PublicKey
    authority: PublicKey
    new_authority: PublicKey


def lease_set_authority(accounts: LeaseSetAuthorityAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(
            pubkey=accounts["new_authority"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xff\x04X\x02\xd5\xafW\x16"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
