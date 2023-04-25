import asyncio
from pytest import fixture, mark

from switchboardpy import (
  SBV2_DEVNET_PID,
  AggregatorOpenRoundParams,
  AggregatorInitParams,
  AggregatorHistoryRow,
  AggregatorAccount,
  AccountParams,
  AggregatorSaveResultParams,
  AggregatorSetHistoryBufferParams,
  OracleQueueAccount
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet

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
    
        agg = AggregatorAccount(AccountParams(program=program, public_key=PublicKey("HMtDNnoCPD6NQRCE2uScEWSvwaZY3hWixCK12TKNtGpc")))

        # getting aggregator data
        data = await agg.load_data()

        assert data.min_oracle_results == 2
        assert data.oracle_request_batch_size == 3
        assert data.min_job_results == 1
        print(data)

@mark.asyncio
async def test_get_latest_value():
    async with SwitchboardProgram() as program:
        agg = AggregatorAccount(AccountParams(program=program, public_key=PublicKey("HMtDNnoCPD6NQRCE2uScEWSvwaZY3hWixCK12TKNtGpc")))

        # getting most recent value
        val = await agg.get_latest_value()
        assert Decimal('39.792') == val

        print('LATEST VALUE:')
        print(val)

@mark.asyncio
async def test_get_latest_timestamp():
    async with SwitchboardProgram() as program:
        agg = AggregatorAccount(AccountParams(program=program, public_key=PublicKey("HMtDNnoCPD6NQRCE2uScEWSvwaZY3hWixCK12TKNtGpc")))

        # getting most recent value
        val = await agg.get_latest_feed_timestamp()
        assert Decimal('1654626799') == val
        print('LATEST TIMESTAMP:')
        print(val)

@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:
        await AggregatorAccount.create(
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
            )
        )