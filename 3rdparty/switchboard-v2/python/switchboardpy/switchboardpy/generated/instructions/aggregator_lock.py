from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class AggregatorLockAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey


def aggregator_lock(accounts: AggregatorLockAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=True),
    ]
    identifier = b"\xe4\xeeC5E\xb0\xb9\xe3"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
