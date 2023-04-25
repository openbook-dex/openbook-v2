import anchorpy

from dataclasses import dataclass
from decimal import Decimal
from typing import Any
from solana import system_program

from solana.keypair import Keypair
from solana.publickey import PublicKey
from spl.token.constants import TOKEN_PROGRAM_ID
from switchboardpy.permission import PermissionAccount
from switchboardpy.program import ProgramStateAccount

from switchboardpy.common import AccountParams
from switchboardpy.oraclequeue import OracleQueueAccount

from .generated.accounts import OracleAccountData

# Parameters for an OracleInit request
@dataclass
class OracleInitParams:
    
    """Specifies the oracle queue to associate with this OracleAccount."""
    queue_account: OracleQueueAccount

    """Buffer specifying orace name"""
    name: bytes = None

    """Buffer specifying oralce metadata"""
    metadata: bytes = None

# Parameters for an OracleWithdraw request.
@dataclass
class OracleWithdrawParams:
    
    """Amount to withdraw"""
    amount: Decimal

    """Token Account to withdraw to"""
    withdraw_account: PublicKey

    """Oracle authority keypair"""
    oracle_authority: Keypair
    
class OracleAccount:
    """ A Switchboard account representing an oracle account and its associated queue
    and escrow account.

    Attributes:
        program (anchor.Program): The anchor program ref
        public_key (PublicKey | None): This aggregator's public key
        keypair (Keypair | None): this aggregator's keypair
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
    Get the size of an OracleAccount on chain

    Args:

    Returns:
        int: size of the OracleAccount type on chain
    """
    def size(self):
        return self.program.account["OracleAccountData"].size

    """
    Load and parse OracleAccount data based on the program IDL

    Args:
    
    Returns:
        OracleAccount

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await OracleAccountData.fetch(self.program.provider.connection, self.public_key)


    """
    Loads a OracleAccount from the expected PDA seed format

    Args:
        program (anchorpy.Program)
        queue_account (OracleQueueAccount)
        wallet (PublicKey)

    Returns:
        Tuple[OracleAccount, int]: OracleAccount and PDA bump
    """
    @staticmethod
    def from_seed(program: anchorpy.Program, queue_account: OracleQueueAccount, wallet: PublicKey):
        oracle_pubkey, bump = PublicKey.find_program_address(
            [
                bytes(b'OracleAccountData'), 
                bytes(queue_account.public_key),
                bytes(wallet),
            ],
            program.program_id
        )
    
        return OracleAccount(AccountParams(program=program, public_key=oracle_pubkey)), bump

    """
    Create and initialize the OracleAccount.

    Args:
        program (anchor.Program): Switchboard program representation holding connection and IDL.
        params (OracleInitParams)
    
    Returns:
        OracleAccount

    """
    @staticmethod
    async def create(program: anchorpy.Program, params: OracleInitParams):
        payer_keypair = Keypair.from_secret_key(program.provider.wallet.payer.secret_key)
        program_state_account, state_bump = ProgramStateAccount.from_seed(program)
        switch_token_mint = await program_state_account.get_token_mint()
        wallet = await switch_token_mint.create_account(program.provider.wallet.public_key)
        await switch_token_mint.set_authority(
            wallet,
            program_state_account.public_key,
            'AccountOwner',
            payer_keypair,
            []
        )
        oracle_account, oracle_bump = OracleAccount.from_seed(
            program,
            params.queue_account,
            wallet
        )

        await program.rpc["oracle_init"](
            {
                "name": params.name or bytes([0] * 32),
                "metadata": params.metadata or bytes([0] * 128),
                "state_bump": state_bump,
                "oracle_bump": oracle_bump,
            },
            ctx=anchorpy.Context(
                accounts={
                    "oracle": oracle_account.public_key,
                    "oracle_authority": payer_keypair.public_key,
                    "queue": params.queue_account.public_key,
                    "wallet": wallet,
                    "program_state": program_state_account.public_key,
                    "system_program": system_program.SYS_PROGRAM_ID,
                    "payer": program.provider.wallet.public_key
                }
            )
        )
        return OracleAccount(AccountParams(program=program, public_key=oracle_account.public_key))

    """
    Inititates a heartbeat for an OracleAccount, signifying oracle is still healthy.

    Args:
    
    Returns:
        TransactionSignature

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def heartbeat(self):
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        oracle = await self.load_data()
        queue_account = OracleQueueAccount(AccountParams(program=self.program,public_key=oracle.queue_pubkey))
        queue_data = await queue_account.load_data()
        last_pubkey = self.public_key
        if queue_data.size != 0:
            last_pubkey = queue_data.queue[queue_data.gc_idx]
        permission_account, permission_bump = PermissionAccount.from_seed(
            self.program,
            queue_data.authority,
            queue_account.public_key,
            self.public_key
        )
        try:
            await permission_account.load_data()
        except Exception:
            raise ValueError('A requested permission pda account has not been initialized.')

        return await self.program.rpc["oracle_heartbeat"](
            {
                "permission_bump": permission_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "oracle": self.public_key,
                    "oracle_authority": payer_keypair.public_key,
                    "token_account": oracle.token_account,
                    "gc_oracle": last_pubkey,
                    "oracle_queue": queue_account.public_key,
                    "permission": permission_account.public_key,
                    "data_buffer": queue_data.data_buffer
                },
                signers=[self.keypair]
            )
        )


    """
    Withdraw stake and/or rewards from an OracleAccount.

    Args:
        params (OracleWithdrawParams)
    
    Returns:
        TransactionSignature

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def withdraw(self, params: OracleWithdrawParams):
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        oracle = await self.load_data()
        queue_pubkey = oracle.queue_pubkey
        queue_account = OracleQueueAccount(AccountParams(program=self.program, public_key=queue_pubkey))
        queue = await queue_account.load_data()
        queue_authority = queue.authority
        state_account, state_bump = ProgramStateAccount.from_seed(self.program)
        permission_account, permission_bump = PermissionAccount.from_seed(
            self.program,
            queue_authority,
            queue_account.public_key,
            self.public_key
        )
        return await self.program.rpc["oracle_withdraw"](
            {
                "permission_bump": permission_bump,
                "state_bump": state_bump,
                "amount": params.amount
            },
            ctx=anchorpy.Context(
                accounts={
                    "oracle": self.public_key,
                    "oracle_authority": params.oracle_authority.public_key,
                    "token_account": oracle.token_account,
                    "withdraw_account": params.withdraw_account,
                    "oracle_queue": queue_account.public_key,
                    "permission": permission_account.public_key,
                    "token_program": TOKEN_PROGRAM_ID,
                    "program_state": state_account.public_key,
                    "system_program": system_program.SYS_PROGRAM_ID,
                    "payer": self.program.provider.wallet.public_key
                },
                signers=[params.oracle_authority]
            )
        )
