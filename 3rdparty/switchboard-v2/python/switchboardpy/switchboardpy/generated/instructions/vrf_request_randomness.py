from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class VrfRequestRandomnessArgs(typing.TypedDict):
    params: types.vrf_request_randomness_params.VrfRequestRandomnessParams


layout = borsh.CStruct(
    "params" / types.vrf_request_randomness_params.VrfRequestRandomnessParams.layout
)


class VrfRequestRandomnessAccounts(typing.TypedDict):
    authority: PublicKey
    vrf: PublicKey
    oracle_queue: PublicKey
    queue_authority: PublicKey
    data_buffer: PublicKey
    permission: PublicKey
    escrow: PublicKey
    payer_wallet: PublicKey
    payer_authority: PublicKey
    recent_blockhashes: PublicKey
    program_state: PublicKey
    token_program: PublicKey


def vrf_request_randomness(
    args: VrfRequestRandomnessArgs, accounts: VrfRequestRandomnessAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=accounts["vrf"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["queue_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["data_buffer"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["escrow"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["payer_wallet"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["payer_authority"], is_signer=True, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["recent_blockhashes"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\xe6y\x0e\xa4\x1c\xdeuv"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
