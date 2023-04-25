import io
import anchorpy

from dataclasses import dataclass
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.system_program import CreateAccountParams, create_account

from switchboardpy.compiled import OracleJob
from switchboardpy.common import AccountParams, parseOracleJob
from switchboardpy.program import ProgramStateAccount

from .generated.accounts import JobAccountData


# Parameters for initializing a JobAccount
@dataclass
class JobInitParams:

    """A serialized protocol buffer holding the schema of the job."""
    data: bytes

    """An optional name to apply to the job account."""
    name: bytes = None

    """unix_timestamp of when funds can be withdrawn from this account."""
    expiration: int = None

    """A required variables oracles must fill to complete the job."""
    variables: list[str] = None

    """A pre-generated keypair to use."""
    keypair: Keypair = None

    """
    An optional wallet for receiving kickbacks from job usage in feeds.
    """
    authority: PublicKey = None

class JobAccount:
    """ A Switchboard account representing a job for an oracle to perform, stored as
        a protocol buffer.

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
        return await JobAccountData.fetch(self.program.provider.connection, self.public_key)


    """
    Load and parse the protobuf from the raw buffer stored in the JobAccount.
    
    Returns:
        OracleJob

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_job(self):
        job = await self.load_data()
        return parseOracleJob(job.data);

    """
    Load and parse JobAccount data based on the program IDL from a buffer.
    
    Args:
        program (anchorpy.Program)
        buf (bytes): Bytes representation of the JobAccount

    Returns:
        Any: JobAccountData parsed in accordance with the
            Switchboard IDL.
    """
    @staticmethod
    def decode(program: anchorpy.Program, buf: bytes):
        coder = anchorpy.Coder(program.idl)
        return coder.accounts.decode(buf)
    
    """
    Create and initialize the JobAccount

    Args:
        program (anchor.Program)
        params (JobInitParams)

    Returns:
        JobAccount
    """
    @staticmethod
    async def create(program: anchorpy.Program, params: JobInitParams):

        job_account = params.keypair or Keypair.generate()
        size = 280 + len(params.data) + (''.join(params.variables) if params.variables else 0)
        state_account, state_bump = ProgramStateAccount.from_seed(program)
        state = await state_account.load_data()
        response = await program.provider.connection.get_minimum_balance_for_rent_exemption(size)
        lamports = response["result"]
        await program.rpc["job_init"](
            {
                "name": params.name or bytes([0] * 32),
                "expiration": params.expiration or 0,
                "data": params.data,
                "variables": [bytes(b'') for _ in params.variables] if params.variables else [],
                "state_bump": state_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "job": job_account.public_key,
                    "authority": params.authority or state.token_vault,
                    "program_state": state_account.public_key
                },
                signers=[job_account],
                pre_instructions=[
                    create_account(
                        CreateAccountParams(
                            from_pubkey=program.provider.wallet.public_key, 
                            new_account_pubkey=job_account.public_key,
                            lamports=lamports, 
                            space=size, 
                            program_id=program.program_id
                        )
                    )
                ]
            )
        )
        return JobAccount(AccountParams(program=program, keypair=job_account))
    