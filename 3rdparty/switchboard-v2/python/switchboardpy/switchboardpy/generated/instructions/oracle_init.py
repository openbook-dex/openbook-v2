from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleInitArgs(typing.TypedDict):
    params: types.oracle_init_params.OracleInitParams


layout = borsh.CStruct("params" / types.oracle_init_params.OracleInitParams.layout)


class OracleInitAccounts(typing.TypedDict):
    oracle: PublicKey
    oracle_authority: PublicKey
    wallet: PublicKey
    program_state: PublicKey
    queue: PublicKey
    payer: PublicKey
    system_program: PublicKey


def oracle_init(
    args: OracleInitArgs, accounts: OracleInitAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["oracle_authority"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["wallet"], is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["payer"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"\x15\x9eBA<\xdd\x94="
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
