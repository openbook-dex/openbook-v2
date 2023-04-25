from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
from ..program_id import PROGRAM_ID


class AggregatorSetAuthorityAccounts(typing.TypedDict):
    aggregator: PublicKey
    authority: PublicKey
    new_authority: PublicKey


def aggregator_set_authority(
    accounts: AggregatorSetAuthorityAccounts,
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(
            pubkey=accounts["new_authority"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\x8c\xb0\x03\xad\x17\x02\x04Q"
    encoded_args = b""
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
