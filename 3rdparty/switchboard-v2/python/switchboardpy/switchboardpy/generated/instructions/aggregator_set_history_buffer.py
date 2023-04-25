from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class AggregatorSetHistoryBufferAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    buffer: PublicKey


def aggregator_set_history_buffer(
    accounts: AggregatorSetHistoryBufferAccounts,
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["buffer"], is_signer=False, is_writable=True),
    ]
    identifier = b"X1\xd6\xf2\xe5,\xab4"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
