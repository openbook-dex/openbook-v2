from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class VrfInitArgs(typing.TypedDict):
    params: types.vrf_init_params.VrfInitParams


layout = borsh.CStruct("params" / types.vrf_init_params.VrfInitParams.layout)


class VrfInitAccounts(typing.TypedDict):
    vrf: PublicKey
    authority: PublicKey
    oracle_queue: PublicKey
    escrow: PublicKey
    program_state: PublicKey
    token_program: PublicKey


def vrf_init(args: VrfInitArgs, accounts: VrfInitAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["vrf"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["oracle_queue"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xf1L\\\xea\xe6\xf0\xa4\x00"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
