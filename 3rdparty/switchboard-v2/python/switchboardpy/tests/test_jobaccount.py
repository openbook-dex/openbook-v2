import asyncio
from pytest import fixture, mark

from switchboardpy import (
    SBV2_DEVNET_PID,
    AccountParams,
    JobAccount,
    JobInitParams,
    OracleJob,
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet
from google.protobuf.internal import encoder

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
          
        job = JobAccount(AccountParams(program=program, public_key=PublicKey("EGn3wTorvMVhnNvGgkr3A8hVXk2aHcsNBCDHjdKMHHtC")))

        # getting aggregator data
        data = await job.load_data()
        print(data)

@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:

        # Jobs protobuf needs to be delimited
        oracleJob = OracleJob()
        task1 = oracleJob.tasks.add()
        httpTask = OracleJob.HttpTask()
        httpTask.url = "https://ftx.us/api/markets/sol/usd"
        task1.http_task.CopyFrom(httpTask)
        task2 = oracleJob.tasks.add()
        parseTask = OracleJob.JsonParseTask()
        parseTask.path = "$.result.price"
        task2.json_parse_task.CopyFrom(parseTask)

        serializedMessage = oracleJob.SerializeToString()
        delimiter = encoder._VarintBytes(len(serializedMessage))

        delimitedOJ = delimiter + serializedMessage

        print(oracleJob.tasks)
        job = await JobAccount.create(
            program=program, 
            params=JobInitParams(
              data=delimitedOJ
            )
        )