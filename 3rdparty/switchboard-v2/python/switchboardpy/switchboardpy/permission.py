import anchorpy

from dataclasses import dataclass
from typing import Any
from enum import Enum
from solana import system_program
from solana.keypair import Keypair
from solana.publickey import PublicKey
from switchboardpy.common import AccountParams

from .generated.accounts import PermissionAccountData

# Parameters for initializing PermissionAccount
@dataclass
class PermissionInitParams:

    """Pubkey of the account granting the permission"""
    granter: PublicKey

    """The receiving amount of a permission"""
    grantee: PublicKey

    """The authority that is allowed to set permissions for this account"""
    authority: PublicKey

# An enum representing all known permission types for Switchboard
class SwitchboardPermission(Enum):
    PERMIT_ORACLE_HEARTBEAT = "permit_oracle_heartbeat"
    PERMIT_ORACLE_QUEUE_USAGE = "permit_oracle_queue_usage"

class SwitchboardPermissionValue(Enum):
    PERMIT_ORACLE_HEARTBEAT = 1 << 0
    PERMIT_ORACLE_QUEUE_USAGE = 1 << 1


# Parameters for setting a permission in a PermissionAccount
@dataclass
class PermissionSetParams:

    """The permission to set"""
    permission: SwitchboardPermission

    """The authority controlling this permission"""
    authority: Keypair

    """Specifies whether to enable or disable the permission"""
    enable: bool

class PermissionAccount:
    """A Switchboard account representing a permission or privilege granted by one
    account signer to another account.

    Attributes:
        program (anchor.Program): The anchor program ref
        public_key (PublicKey | None): This permission's public key
        keypair (Keypair | None): this permission's keypair
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
    Check if a specific permission is enabled on this permission account

    Args:
        permission (SwitchboardPermissionValue)

    Returns:
        bool: whether or not the permission is enabled
    """
    async def is_permission_enabled(self, permission: SwitchboardPermissionValue):
        perm_data = await self.load_data()
        permissions = perm_data.permissions
        return (permissions & permission) != 0

    """
    Load and parse PermissionAccount data based on the program IDL

    Args:
    
    Returns:
        PermissionAccount

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await PermissionAccountData.fetch(self.program.provider.connection, self.public_key)


    """
    Get the size of a PermissionAccount on chain

    Args:

    Returns:
        int: size of the PermissionAccount type on chain
    """
    def size(self):
        return self.program.account["PermissionAccountData"].size

    """
    Create and initialize a PermissionAccount

    Args:
        program (anchor.Program)
        prarams (PermissionInitParams)

    Returns:
        PermissionAccount
    """
    @staticmethod
    async def create(program: anchorpy.Program, params: PermissionInitParams):
        permission_account, permission_bump = PermissionAccount.from_seed(
            program,
            params.authority,
            params.granter,
            params.grantee
        )

        await program.rpc["permission_init"](
            {
                "permission_bump": permission_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "permission": permission_account.public_key,
                    "authority": params.authority,
                    "granter": params.granter,
                    "grantee": params.grantee,
                    "system_program": system_program.SYS_PROGRAM_ID,
                    "payer": program.provider.wallet.public_key
                },
            )
        )
        return permission_account

    """
    Loads a PermissionAccount from the expected PDA seed format

    Args:
        program (anchorpy.Program)
        authority (public_key): The authority pubkey to be incorporated into the account seed.
        granter (public_key): The granter pubkey to be incorporated into the account seed.
        grantee (public_key): The grantee pubkey to be incorporated into the account seed.

    Returns:
        Tuple[PermissionAccount, int]: PermissionAccount and PDA bump
    """
    @staticmethod
    def from_seed(program: anchorpy.Program, authority: PublicKey, granter: PublicKey, grantee: PublicKey):
        pubkey, bump = PublicKey.find_program_address(
            [
                bytes(b'PermissionAccountData'), 
                bytes(authority),
                bytes(granter),
                bytes(grantee)
            ],
            program.program_id
        )
    
        return PermissionAccount(AccountParams(program=program, public_key=pubkey)), bump

    """
    Sets the permission in the PermissionAccount

    Args: 
        params (PermissionSetParams)
    
    Returns:
        TransactionSignature
    """
    async def set(self, params: PermissionSetParams):
        self.program.rpc["permission_set"](
            {
                "permission": self.program.type["SwitchboardPermission"][params.permission](),
                "authority": params.authority.public_key
            },
            ctx=anchorpy.Context(
                accounts={
                    "permission": self.public_key,
                    "authority": params.authority.public_key
                },
                signers=[params.authority]
            )
        )