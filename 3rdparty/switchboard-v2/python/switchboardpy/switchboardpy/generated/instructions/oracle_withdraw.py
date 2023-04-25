from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleWithdrawArgs(typing.TypedDict):
    params: types.oracle_withdraw_params.OracleWithdrawParams


layout = borsh.CStruct(
    "params" / types.oracle_withdraw_params.OracleWithdrawParams.layout
)


class OracleWithdrawAccounts(typing.TypedDict):
    oracle: PublicKey
    oracle_authority: PublicKey
    token_account: PublicKey
    withdraw_account: PublicKey
    oracle_queue: PublicKey
    permission: PublicKey
    token_program: PublicKey
    program_state: PublicKey
    payer: PublicKey
    system_program: PublicKey


def oracle_withdraw(
    args: OracleWithdrawArgs, accounts: OracleWithdrawAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["oracle_authority"], is_signer=True, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_account"], is_signer=False, is_writable=True
        ),
        AccountMeta(
            pubkey=accounts["withdraw_account"], is_signer=False, is_writable=True
        ),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["token_program"], is_signer=False, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["program_state"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["payer"], is_signer=True, is_writable=True),
        AccountMeta(
            pubkey=accounts["system_program"], is_signer=False, is_writable=False
        ),
    ]
    identifier = b"+\x04\xc8\x84`\x96|0"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
