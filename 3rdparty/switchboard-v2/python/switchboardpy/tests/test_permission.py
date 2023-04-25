import asyncio
from pytest import fixture, mark

from switchboardpy import (
  SBV2_DEVNET_PID,
  AccountParams,
  PermissionAccount,
  PermissionInitParams,
  PermissionSetParams,
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
      provider = Provider(client, Wallet(Keypair()))
      self.program = await Program.at(
          SBV2_DEVNET_PID, provider
      )
      return self.program
    
    async def __aexit__(self, exc_t, exc_v, exc_tb):
        await self.program.close()

@mark.asyncio
async def test_load_data():
    async with SwitchboardProgram() as program:
          
        permission = PermissionAccount(AccountParams(program=program, public_key=PublicKey("9BBmwkQpmoy3XERYHdeV6uL2bub9C3UCWxfxDwdstDDT")))

        # getting aggregator data
        data = await permission.load_data()
        print(data)
