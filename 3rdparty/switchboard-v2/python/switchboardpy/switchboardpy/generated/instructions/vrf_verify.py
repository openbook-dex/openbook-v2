from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class VrfVerifyArgs(typing.TypedDict):
    params: types.vrf_verify_params.VrfVerifyParams


layout = borsh.CStruct("params" / types.vrf_verify_params.VrfVerifyParams.layout)


class VrfVerifyAccounts(typing.TypedDict):
    vrf: PublicKey
    callback_pid: PublicKey
    token_program: PublicKey
    escrow: PublicKey
    program_state: PublicKey
    oracle: PublicKey
    oracle_authority: PublicKey
    oracle_wallet: PublicKey
    instructions_sysvar: PublicKey


def vrf_verify(
    args: VrfVerifyArgs, accounts: VrfVerifyAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["vrf"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["callback_pid"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["oracle_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["oracle_wallet"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["instructions_sysvar"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"5e\r\x1e\xf5\xd5f\x96"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
