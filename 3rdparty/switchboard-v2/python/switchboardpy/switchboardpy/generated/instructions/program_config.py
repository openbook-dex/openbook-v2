from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class ProgramConfigArgs(typing.TypedDict):
    params: types.program_config_params.ProgramConfigParams


layout = borsh.CStruct(
    "params" / types.program_config_params.ProgramConfigParams.layout
)


class ProgramConfigAccounts(typing.TypedDict):
    authority: PublicKey
    program_state: PublicKey


def program_config(
    args: ProgramConfigArgs, accounts: ProgramConfigAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b">{\x14\x968m\xd1\x91"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
