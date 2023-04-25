from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class AggregatorAddJobAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    job: PublicKey


def aggregator_add_job(accounts: AggregatorAddJobAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["job"], is_signer=False, is_writable=True),
    ]
    identifier = b"\x84\x1e#3s\x8e\xba\n"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
