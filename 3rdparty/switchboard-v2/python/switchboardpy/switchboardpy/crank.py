import anchorpy
import math
import time

from dataclasses import dataclass
from typing import Any, Dict, Optional
from solana import system_program

from solana.keypair import Keypair
from solana.publickey import PublicKey
from spl.token.constants import TOKEN_PROGRAM_ID
from spl.token.instructions import get_associated_token_address
from switchboardpy.lease import LeaseAccount
from switchboardpy.permission import PermissionAccount
from switchboardpy.common import AccountParams
from switchboardpy.oraclequeue import OracleQueueAccount
from switchboardpy.aggregator import AggregatorAccount
from solana.system_program import CreateAccountParams, create_account

from switchboardpy.program import ProgramStateAccount

from .generated.accounts import CrankAccountData

# Parameters for initializing a CrankAccount
@dataclass
class CrankInitParams:

    """OracleQueueAccount for which this crank is associated"""
    queue_account: OracleQueueAccount

    """Buffer specifying crank name"""
    name: bytes = None

    """Buffer specifying crank metadata"""
    metadata: bytes = None

    """Optional max number of rows"""
    max_rows: int = None

# Parameters for popping an element from a CrankAccount
@dataclass
class CrankPopParams:

    """Specifies the wallet to reward for turning the crank."""
    payout_wallet: PublicKey

    """The pubkey of the linked oracle queue."""
    queue_pubkey: PublicKey

    """The pubkey of the linked oracle queue authority."""
    queue_authority: PublicKey

    """CrankAccount data"""
    crank: Any

    """QueueAccount data"""
    queue: Any

    """Token mint pubkey"""
    token_mint: PublicKey

    """
    Array of pubkeys to attempt to pop. If discluded, this will be loaded
    from the crank upon calling.
    """
    ready_pubkeys: list[PublicKey] = None

    """Nonce to allow consecutive crank pops with the same blockhash."""
    nonce: int = None
    fail_open_on_mismatch: bool = None

# Parameters for pushing an element into a CrankAccount
@dataclass
class CrankPushParams:
    aggregator_account: AggregatorAccount

# Row structure of elements in the crank
@dataclass
class CrankRow:

    """Aggregator account pubkey"""
    pubkey: PublicKey

    """Next aggregator update timestamp to order the crank by"""
    next_timestamp: int

    @staticmethod
    def from_bytes(buf: bytes):
        pass

class CrankAccount:
    """ A Switchboard account representing a crank of aggregators ordered by next update time.

    Attributes:
        program (anchor.Program): The anchor program ref
        public_key (PublicKey | None): This crank's public key
        keypair (Keypair | None): this crank's keypair
    """


    def __init__(self, params: AccountParams):
        if params.public_key is None and params.keypair is None:
            raise ValueError('User must provide either a publicKey or keypair for account use.')
        if params.keypair and params.public_key and params.keypair.public_key != params.public_key:
            raise ValueError('User must provide either a publicKey or keypair for account use.')
        self.program = params.program
        self.public_key = params.keypair.public_key if params.keypair else params.public_key
        self.keypair = params.keypair
    
    """
    Get the size of an CrankAccount on chain

    Args:

    Returns:
        int: size of the CrankAccount type on chain
    """
    def size(self):
        return self.program.account["CrankAccountData"].size

    """
    Load and parse CrankAccount data based on the program IDL

    Args:
    
    Returns:
        CrankAccount

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await CrankAccountData.fetch(self.program.provider.connection, self.public_key)


    """
    Create and initialize the CrankAccount.

    Args:
        program (anchor.Program): Switchboard program representation holding connection and IDL.
        params (CrankInitParams)
    
    Returns:
        CrankAccount
    """
    @staticmethod
    async def create(program: anchorpy.Program, params: CrankInitParams):
        crank_account = Keypair.generate()
        buffer = Keypair.generate()
        size = program.account["CrankAccountData"].size
        max_rows = params.max_rows or 500
        crank_size = max_rows * 40 + 8
        response = await program.provider.connection.get_minimum_balance_for_rent_exemption(crank_size)
        lamports = response["result"]
        await program.rpc["crank_init"](
            {
                "name": params.name or bytes([0] * 32),
                "metadata": params.metadata or bytes([0] * 128),
                "crank_size": max_rows
            },
            ctx=anchorpy.Context(
                accounts={
                    "crank": crank_account.public_key,
                    "queue": params.queue_account.public_key,
                    "buffer": buffer.public_key,
                    "system_program": system_program.SYS_PROGRAM_ID,
                    "payer": program.provider.wallet.public_key
                },
                signers=[crank_account, buffer],
                pre_instructions=[
                    create_account(
                        CreateAccountParams(
                            from_pubkey=program.provider.wallet.public_key, 
                            new_account_pubkey=buffer.public_key,
                            lamports=lamports, 
                            space=size, 
                            program_id=program.program_id
                        )
                    )
                ]
            )
        )

        return CrankAccount(AccountParams(program=program, keypair=crank_account))

    """
    Pushes a new aggregator onto the crank
    
    Args:
        params (CrankPushParams): aggregator and related data
    
    Returns:
        TransactionSignature
    """
    async def push(self, params: CrankPushParams):
        aggregator_account: AggregatorAccount = params.aggregator_account
        crank = await self.load_data()
        queue_account = OracleQueueAccount(AccountParams(program=self.program, public_key=crank.queue_pubkey))
        queue = await queue_account.load_data()
        queue_authority = queue.authority
        lease_account, lease_bump = LeaseAccount.from_seed(self.program, queue_account, aggregator_account)
        lease: Any = None
        try:
            lease = await lease_account.load_data()
        except Exception:
            raise ValueError('A requested lease pda account has not been initialized.')
        permission_account, permission_bump = PermissionAccount.from_seed(
            self.program,
            queue_authority,
            queue_account.public_key,
            aggregator_account.public_key
        )
        try:
            await lease_account.load_data()
        except Exception:
            raise ValueError('A requested permission pda account has not been initialized.')
        program_state_account, state_bump = ProgramStateAccount.from_seed(self.program)
        return await self.program.rpc["crank_push"](
            {
                "state_bump": state_bump,
                "permission_bump": permission_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "crank": self.public_key,
                    "aggregator": aggregator_account.public_key,
                    "oracle_queue": queue_account.public_key,
                    "queue_authority": queue_authority,
                    "permission": permission_account.public_key,
                    "lease": lease_account.public_key,
                    "escrow": lease.escrow,
                    "program_state": program_state_account.public_key,
                    "data_buffer": crank.data_buffer
                }
            )
        )


    """
    Pops a tx from the crank.

    Args:
        params (CrankPopParams)

    Returns:
        TransactionSignature    
    """
    async def pop_txn(self, params: CrankPopParams):
        fail_open_on_account_mismatch = params.fail_open_on_mismatch or False
        next = params.ready_pubkeys or await self.peak_next_ready(5)
        if len(next) == 0:
            raise ValueError('Crank is not ready to be turned')
        remaining_accounts: list[PublicKey] = []
        lease_bumps_map: Dict[str, int] = {}
        permission_bumps_map: Dict[str, int] = {}
        queue_account = OracleQueueAccount(AccountParams(program=self.program, public_key=params.queue_pubkey))
        for row in next:
            aggregator_account = AggregatorAccount(AccountParams(program=self.program, public_key=row))
            lease_account, lease_bump = LeaseAccount.from_seed(
                self.program,
                queue_account,
                aggregator_account
            )
            permission_account, permission_bump = PermissionAccount.from_seed(
                self.program,
                params.queue_authority,
                params.queue_pubkey,
                row
            )
            escrow = get_associated_token_address(
                lease_account.public_key,
                params.token_mint
            )
            remaining_accounts.append(aggregator_account.public_key)
            remaining_accounts.append(lease_account.public_key)
            remaining_accounts.append(escrow)
            remaining_accounts.append(permission_account.public_key)
            lease_bumps_map[row.to_base58()] = lease_bump
            permission_bumps_map[row.to_base58()] = permission_bump
        remaining_accounts.sort(key=lambda key : bytes(key))
        crank = params.crank
        queue = params.queue
        lease_bumps: list[int] = []
        permission_bumps: list[int] = []
        for key in remaining_accounts:
            lease_bumps.append(lease_bumps_map.get(key.to_base58()) or 0)
            permission_bumps.append(permission_bumps_map.get(key.to_base58()) or 0)
        program_state_account, state_bump = ProgramStateAccount.from_seed(self.program)
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        return self.program.transaction["crank_pop"](
            {
                "state_bump": state_bump,
                "lease_bumps": bytes(lease_bumps),
                "permission_bumps": bytes(permission_bumps),
                "nonce": params.nonce or None,
                "fail_open_on_account_mismatch": fail_open_on_account_mismatch
            },
            ctx=anchorpy.Context(
                accounts={
                    "crank": self.public_key,
                    "oracle_queue": params.queue_pubkey,
                    "queue_authority": params.queue_authority,
                    "program_state": program_state_account.public_key,
                    "payout_wallet": params.payout_wallet,
                    "token_program": TOKEN_PROGRAM_ID,
                    "crank_data_buffer": crank.data_buffer,
                    "queue_data_buffer": queue.data_buffer
                },
                remaining_accounts=[{ "is_signer": False, "is_writable": True, "pubkey": pubkey } for pubkey in remaining_accounts],
                signers=[payer_keypair]
            )
        )

    """
    Pops an aggregator from the crank

    Args:
        params (CrankPopParams)
    
    Returns:
        TransactionSignature
    """
    async def pop(self, params: CrankPopParams):
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        txn = await self.pop_txn(params)
        return await self.program.provider.connection.send_transaction(txn, [payer_keypair])
    
    """
    Get an array of the next aggregator pubkeys to be popped from the crank, limited by n

    Args:
        n (int): limit of pubkeys to return

    Returns:
        list[CrankRow]: Pubkey list of Aggregators and next timestamp to be popped, ordered by timestamp
    """
    async def peak_next_with_time(self, n: int):
        crank = await self.load_data()

        # get list slice of length pq_size 
        pq_data: list[CrankRow] = crank.pq_data[:crank.pq_size]

        # sort by CrankRow next timestamp
        pq_data.sort(key=lambda crank_row: crank_row.next_timestamp)

        # return items
        return pq_data[:n]

    """
    Get an array of the next readily updateable aggregator pubkeys to be popped
    from the crank, limited by n

    Args:
        n (Optional[int]): limit of pubkeys to return

    Returns:
        list[PublicKey]: Pubkey list of Aggregators and next timestamp to be popped, ordered by timestamp
    """
    async def peak_next_ready(self, n: Optional[int] = None):
        now = math.floor(time.time())
        crank = await self.load_data()
        pq_data: list[CrankRow] = crank.pq_data[:crank.pq_size]
        key = lambda crank_row: crank_row.next_timestamp
        return [item.pubkey for item in list(filter(lambda item: now >= item.next_timestamp, pq_data)).sort(key=key)[:(n or len(pq_data))]]
        
    """
    Get an array of the next aggregator pubkeys to be popped from the crank, limited by n

    Args:
        n (int): limit of pubkeys to return

    Returns:
        list[PublicKey]: Pubkey list of Aggregators and next timestamp to be popped, ordered by timestamp
    """
    async def peak_next(self, n: int):
        crank = await self.load_data()
        pq_data: list[CrankRow] = crank.pq_data[:crank.pq_size]
        pq_data.sort(key=lambda crank_row: crank_row.next_timestamp)
        return [item.pubkey for item in pq_data[:n]]