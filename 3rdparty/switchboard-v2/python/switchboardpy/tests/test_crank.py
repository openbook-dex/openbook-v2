import asyncio
from pytest import fixture, mark

from switchboardpy import (
  SBV2_DEVNET_PID,
  AccountParams,
  CrankAccount,
  CrankPopParams,
  CrankPushParams,
  CrankInitParams,
  CrankRow,
  OracleQueueAccount
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet

CRANK_DEVNET = 'GN9jjCy2THzZxhYqZETmPM3my8vg4R5JyNkgULddUMa5' #  <-- new key | old key 'HX2oLYGqThai8i6hvEm9B4y5pAkLXLyryps13195BSAz';

class SwitchboardProgram(object):

    async def __aenter__(self):
      client = AsyncClient("https://api.devnet.solana.com/")
      provider = Provider(client, Wallet.local()) # 2RBU9Eie9GpBe8kY81Vo3zHwnXMBbcvh8bnb6f9CLzts
      self.program = await Program.at(
          SBV2_DEVNET_PID, provider
      )
      return self.program
    
    async def __aexit__(self, exc_t, exc_v, exc_tb):
        await self.program.close()

@mark.asyncio
async def test_load_data():
    async with SwitchboardProgram() as program:
          
        crank = CrankAccount(AccountParams(program=program, public_key=PublicKey(CRANK_DEVNET)))

        # getting aggregator data
        data = await crank.load_data()
        print(data)

@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:
        """
        crank = await CrankAccount.create(
            program=program, 
            params=CrankInitParams(
                queue_account=OracleQueueAccount(
                    AccountParams(
                        program=program, 
                        public_key=PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy")
                    )
                ),
            )
        )
        """