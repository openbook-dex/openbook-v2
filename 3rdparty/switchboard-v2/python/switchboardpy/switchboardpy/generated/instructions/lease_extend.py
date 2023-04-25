from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class LeaseExtendArgs(typing.TypedDict):
    params: types.lease_extend_params.LeaseExtendParams


layout = borsh.CStruct("params" / types.lease_extend_params.LeaseExtendParams.layout)


class LeaseExtendAccounts(typing.TypedDict):
    lease: PublicKey
    aggregator: PublicKey
    queue: PublicKey
    funder: PublicKey
    owner: PublicKey
    escrow: PublicKey
    token_program: PublicKey
    program_state: PublicKey


def lease_extend(
    args: LeaseExtendArgs, accounts: LeaseExtendAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["funder"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["owner"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xcaF\x8d\x1d\x88\x8e\xe6v"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
