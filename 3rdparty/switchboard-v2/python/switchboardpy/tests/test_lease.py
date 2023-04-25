import asyncio
from pytest import fixture, mark

from switchboardpy import (
  SBV2_DEVNET_PID,
  AccountParams,
  LeaseAccount,
  LeaseWithdrawParams,
  LeaseExtendParams,
  LeaseInitParams,
  AggregatorAccount,
  AggregatorInitParams,
  OracleQueueAccount
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet
from spl.token.async_client import AsyncToken
from spl.token.constants import TOKEN_PROGRAM_ID

from switchboardpy.aggregator import AggregatorAccount

class SwitchboardProgram(object):

    async def __aenter__(self):
      client = AsyncClient("https://api.devnet.solana.com/", commitment="confirmed")
      provider = Provider(client, Wallet.local())
      self.program = await Program.at(
          SBV2_DEVNET_PID, provider
      )
      return self.program
    
    async def __aexit__(self, exc_t, exc_v, exc_tb):
        await self.program.close()

@mark.asyncio
async def test_load_data():
    async with SwitchboardProgram() as program:
          
        lease = LeaseAccount(AccountParams(program=program, public_key=PublicKey("qAs3FQX2iUSRCe9WFXbRgH594LSqusTUze8BftxbiHC")))

        # getting aggregator data
        data = await lease.load_data()
        print(data)

@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:
        """
        # create aggregator so we can later make a lease for it
        aggregator = await AggregatorAccount.create(
            program=program, 
            aggregator_init_params=AggregatorInitParams(
                batch_size=3, 
                min_required_oracle_results=2, 
                min_required_job_results=1, 
                min_update_delay_seconds=300, 
                queue_account=OracleQueueAccount(
                    AccountParams(
                        program=program, 
                        public_key=PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy")
                    )
                ),
                start_after=0,
            )
        )

        # create tokenAccount to fund lease
        tokenAccount = await AsyncToken.create_wrapped_native_account(
            program.provider.connection, 
            TOKEN_PROGRAM_ID, 
            program.provider.wallet.public_key, 
            program.provider.wallet.payer, 
            5000, # load with 5000 lamps
            skip_confirmation=False
        )

        # create lease
        lease = await LeaseAccount.create(
            program=program, 
            params=LeaseInitParams(
                withdraw_authority=program.provider.wallet.public_key,
                load_amount=10, 
                funder=tokenAccount,
                funder_authority=program.provider.wallet.payer,
                aggregator_account=aggregator,
                oracle_queue_account=OracleQueueAccount(
                    AccountParams(
                        program=program, 
                        public_key=PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy")
                    )
                ),
            )
        )"""