from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleHeartbeatArgs(typing.TypedDict):
    params: types.oracle_heartbeat_params.OracleHeartbeatParams


layout = borsh.CStruct(
    "params" / types.oracle_heartbeat_params.OracleHeartbeatParams.layout
)


class OracleHeartbeatAccounts(typing.TypedDict):
    oracle: PublicKey
    oracle_authority: PublicKey
    token_account: PublicKey
    gc_oracle: PublicKey
    oracle_queue: PublicKey
    permission: PublicKey
    data_buffer: PublicKey


def oracle_heartbeat(
    args: OracleHeartbeatArgs, accounts: OracleHeartbeatAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["oracle"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["oracle_authority"], is_signer=True, is_writable=False
        ),
        AccountMeta(
            pubkey=accounts["token_account"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["gc_oracle"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["oracle_queue"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["permission"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["data_buffer"], is_signer=False, is_writable=True),
    ]
    identifier = b"\n\xaf\xd9\x82o#u6"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
