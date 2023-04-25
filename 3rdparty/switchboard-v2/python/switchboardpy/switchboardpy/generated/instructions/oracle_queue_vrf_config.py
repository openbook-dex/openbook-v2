from __future__ import annotations
import typing
from solana.publickey import PublicKey
from solana.transaction import TransactionInstruction, AccountMeta
import borsh_construct as borsh
from .. import types
from ..program_id import PROGRAM_ID


class OracleQueueVrfConfigArgs(typing.TypedDict):
    params: types.oracle_queue_vrf_config_params.OracleQueueVrfConfigParams


layout = borsh.CStruct(
    "params" / types.oracle_queue_vrf_config_params.OracleQueueVrfConfigParams.layout
)


class OracleQueueVrfConfigAccounts(typing.TypedDict):
    queue: PublicKey
    authority: PublicKey


def oracle_queue_vrf_config(
    args: OracleQueueVrfConfigArgs, accounts: OracleQueueVrfConfigAccounts
) -> TransactionInstruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["queue"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["authority"], is_signer=True, is_writable=False),
    ]
    identifier = b"WS\xb5\xa1\x95>\x83\x7f"
    encoded_args = layout.build(
        {
            "params": args["params"].to_encodable(),
        }
    )
    data = identifier + encoded_args
    return TransactionInstruction(keys, PROGRAM_ID, data)
