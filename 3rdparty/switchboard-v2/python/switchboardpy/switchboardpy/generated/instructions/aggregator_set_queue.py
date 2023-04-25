from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class AggregatorSetQueueAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    queue: PublicKey


def aggregator_set_queue(
    accounts: AggregatorSetQueueAccounts,
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
    ]
    identifier = b"o\x98\x8e\x99\xce'\x16\x94"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
