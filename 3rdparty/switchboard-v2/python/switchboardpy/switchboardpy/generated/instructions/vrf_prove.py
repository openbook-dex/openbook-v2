from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class VrfProveArgs(typing.TypedDict):
    params: types.vrf_prove_params.VrfProveParams


layout = borsh.CStruct("params" / types.vrf_prove_params.VrfProveParams.layout)


class VrfProveAccounts(typing.TypedDict):
    vrf: PublicKey
    oracle: PublicKey
    randomness_producer: PublicKey


def vrf_prove(args: VrfProveArgs, accounts: VrfProveAccounts) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["vrf"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["randomness_producer"], is_signer=True, is_writable=False
        ),
    ]
    identifier = b"\x83^\xca\xbf\xcfy)\xea"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
