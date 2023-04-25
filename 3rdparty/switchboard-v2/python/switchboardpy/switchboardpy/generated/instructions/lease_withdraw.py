from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class LeaseWithdrawArgs(typing.TypedDict):
    params: types.lease_withdraw_params.LeaseWithdrawParams


layout = borsh.CStruct(
    "params" / types.lease_withdraw_params.LeaseWithdrawParams.layout
)


class LeaseWithdrawAccounts(typing.TypedDict):
    lease: PublicKey
    escrow: PublicKey
    aggregator: PublicKey
    queue: PublicKey
    withdraw_authority: PublicKey
    withdraw_account: PublicKey
    token_program: PublicKey
    program_state: PublicKey


def lease_withdraw(
    args: LeaseWithdrawArgs, accounts: LeaseWithdrawAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["withdraw_authority"], is_signer=True, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["withdraw_account"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xba)d\xf8\xeaQ=\xa9"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
