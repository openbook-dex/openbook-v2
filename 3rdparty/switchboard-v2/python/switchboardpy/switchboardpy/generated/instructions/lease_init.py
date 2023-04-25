from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class LeaseInitArgs(typing.TypedDict):
    params: types.lease_init_params.LeaseInitParams


layout = borsh.CStruct("params" / types.lease_init_params.LeaseInitParams.layout)


class LeaseInitAccounts(typing.TypedDict):
    lease: PublicKey
    queue: PublicKey
    aggregator: PublicKey
    funder: PublicKey
    payer: PublicKey
    system_program: PublicKey
    token_program: PublicKey
    owner: PublicKey
    escrow: PublicKey
    program_state: PublicKey


def lease_init(
    args: LeaseInitArgs, accounts: LeaseInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["lease"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["aggregator"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["funder"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["payer"], is_signer=True, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["owner"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xa8\xbe\x9d\xfc\x9f\xe2\xf1Y"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
