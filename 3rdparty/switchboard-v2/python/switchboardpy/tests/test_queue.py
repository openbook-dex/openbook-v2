import asyncio
from pytest import fixture, mark

from switchboardpy import (
  SBV2_DEVNET_PID,
  AccountParams,
  OracleQueueAccount,
  OracleQueueInitParams,
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet
from switchboardpy.program import ProgramStateAccount 

ORACLE_QUEUE_STANDARD_DEVNET = 'F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy' # <-- new key | old key - 'B4yBQ3hYcjnrNLxUnauJqwpFJnjtm7s8gHybgkAdgXhQ';


class SwitchboardProgram(object):

    async def __aenter__(self):
      client = AsyncClient("https://api.devnet.solana.com/")
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
          
        queue = OracleQueueAccount(AccountParams(program=program, public_key=PublicKey(ORACLE_QUEUE_STANDARD_DEVNET)))

        # getting aggregator data
        data = await queue.load_data()
        print(data)
        
@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:
        program_state_account, state_bump = ProgramStateAccount.from_seed(program)
        switch_token_mint = await program_state_account.get_token_mint()
        await OracleQueueAccount.create(
            program=program, 
            params=OracleQueueInitParams(
                reward=3000,
                min_stake=300, 
                authority=program.provider.wallet.public_key, #
                oracle_timeout=20000,
                mint=switch_token_mint.pubkey
            )
        )