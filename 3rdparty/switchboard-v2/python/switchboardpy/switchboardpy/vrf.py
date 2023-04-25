import anchorpy

from dataclasses import dataclass
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.transaction import Transaction
from solana.system_program import CreateAccountParams, create_account
from solana.rpc.commitment import Confirmed
from switchboardpy.common import AccountParams

from generated.accounts import VrfAccountData
from generated.instructions import VrfInitArgs, VrfInitAccounts, vrf_init

@dataclass
class VrfInitParams:
    
    """Generated VrfInitArgs"""
    vrf_init_args: VrfInitArgs

    """Buffer specifying orace name"""
    vrf_init_accounts: VrfInitAccounts

class VrfAccount:
    """ A Switchboard account representing a VrfAccount

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
    Load and parse JobAccount state based on the program IDL. 
    
    Returns:
        name (JobAccount): data parsed in accordance with the
            Switchboard IDL.

    Args:

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await VrfAccountData.fetch(self.program.provider.connection, self.public_key)


    """
    Create and initialize the VrfAccount

    Args:
        program (anchor.Program)
        params (VrfInitArgs)

    Returns:
        VrfAccount
    """
    @staticmethod
    async def create(program: anchorpy.Program, params: VrfInitParams):
        ix = vrf_init(params.vrf_init_args, params.vrf_init_accounts)
        tx = Transaction().add(ix)    
        sig = await program.provider.send(tx)
        await program.provider.connection.confirm_transaction(sig, Confirmed)
        return VrfAccount(AccountParams(program=program, public_key=params.vrf_init_accounts.vrf))


    