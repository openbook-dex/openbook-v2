import anchorpy

from dataclasses import dataclass
from decimal import Decimal
from solana import publickey
from solana import system_program

from spl.token.async_client import AsyncToken
from spl.token.constants import TOKEN_PROGRAM_ID
from solana.keypair import Keypair
from solana.publickey import PublicKey

from switchboardpy.common import AccountParams

from .generated.accounts import SbState

# Devnet Program ID.
SBV2_DEVNET_PID = PublicKey(
    '2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG'
)

# Mainnet-Beta Program ID.
SBV2_MAINNET_PID = PublicKey(
    'SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f'
)


# Input parameters intitializing program state
@dataclass
class ProgramInitParams:

    """Optional token mint"""
    mint: PublicKey = None

# Input parameters for transferring from Switchboard to token vault
@dataclass
class VaultTransferParams:

    """Amount being transferred"""
    amount: Decimal

class ProgramStateAccount:
    """Account type representing Switchboard global program state.

    Attributes:
        program (anchor.Program): The anchor program ref
        public_key (PublicKey | None): This program's public key
        keypair (Keypair | None): this program's keypair
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
    Constructs ProgramStateAccount from the static seed from which it was generated.

    Args:
        program (anchorpy.Program): Anchor-loaded aggregator

    Returns:
        ProgramStateAccount and PDA bump tuple.
    """
    @staticmethod
    def from_seed(program: anchorpy.Program):
        state_pubkey, state_bump = publickey.PublicKey.find_program_address(['STATE'.encode()], program.program_id)
        return ProgramStateAccount(AccountParams(program=program, public_key=state_pubkey)), state_bump

    """
    Load and parse ProgramStateAccount state based on the program IDL. 
    
    Args:

    Returns:
        name (Any): data parsed in accordance with the
            Switchboard IDL.

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await SbState.fetch(self.program.provider.connection, self.public_key)


    """
    Fetch the Switchboard token mint specified in the program state account.
    
    Args:

    Returns:
        anchorpy.
    """
    async def get_token_mint(self) -> AsyncToken:
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        state = await self.load_data()
        switch_token_mint = AsyncToken(self.program.provider.connection, state.token_mint, TOKEN_PROGRAM_ID, payer_keypair)
        return switch_token_mint

    """
    Get the size of the global ProgramStateAccount on chain
    
    Returns:
        int: size of the ProgramStateAccount on chain 
    """
    def size(self):
        return self.program.account["SbState"].size

    """
    Create and initialize the ProgramStateAccount

    Args:
        program (anchorpy.Program): anchor program
        params (ProgramInitParams): optionally pass in mint address

    Returns:
        ProgramStateAccount that was generated
    """
    @staticmethod
    async def create(program: anchorpy.Program, params: ProgramInitParams):
        payer_keypair = Keypair.from_secret_key(program.provider.wallet.payer.secret_key)
        state_account, state_bump = ProgramStateAccount.from_seed(program)
        psa = ProgramStateAccount(AccountParams(program=program, public_key=state_account.public_key))
        try:
            await psa.load_data()
            return psa
        except Exception:
            pass
        mint = None
        vault = None
        if params.mint == None:
            decimals = 9
            mint, vault = await anchorpy.utils.token.create_mint_and_vault(
                program.provider,
                100_000_000,
                payer_keypair.public_key,
                decimals
            )
        else:
            mint = params.mint
            token = AsyncToken(
                program.provider.connection,
                mint,
                TOKEN_PROGRAM_ID,
                payer_keypair
            )
            vault = await token.create_account(payer_keypair.public_key)
        await program.rpc["program_init"](
            {
                "state_bump": state_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "state": state_account.public_key,
                    "authority": payer_keypair.public_key,
                    "token_mint": mint,
                    "vault": vault,
                    "payer": payer_keypair.public_key,
                    "system_program": system_program.SYS_PROGRAM_ID,
                    "token_program": TOKEN_PROGRAM_ID
                },
            )
        )

    """
    Transfer N tokens from the program vault to a specified account.

    Args:
        to (PublicKey): The recipient of the vault tokens.
        authority (Keypair): The vault authority required to sign the transfer tx
        params (VaultTransferParams): Specifies the amount to transfer.
  
    Returns:
        TransactionSignature
    """
    async def vault_transfer(self, to: PublicKey, authority: Keypair, params: VaultTransferParams):
        state_pubkey, state_bump = ProgramStateAccount.from_seed(self.program)
        state = await self.load_data()
        vault = state.token_vault
        await self.program.rpc["vault_transfer"](
            {
                "state_bump": state_bump,
                "amount": params.amount # @FIXME - can't be a decimal, must have mantissa / scale
            },
            ctx=anchorpy.Context(
                accounts={
                    "state": state_pubkey,
                    "to": to,
                    "vault": vault,
                    "authority": authority.public_key,
                    "token_program": TOKEN_PROGRAM_ID,
                },
                signers=[authority]
            )
        )
