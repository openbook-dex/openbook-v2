import asyncio
from pytest import fixture, mark

from switchboardpy import (
    SBV2_DEVNET_PID,
    AccountParams,
    LeaseAccount,
    LeaseInitParams,
    AggregatorAccount,
    AggregatorInitParams,
    OracleQueueAccount,
    JobAccount,
    JobInitParams,
    CrankAccount,
    CrankPushParams,
    PermissionAccount,
    PermissionInitParams,
    OracleJob,
)

from contextlib import contextmanager
from decimal import Decimal
from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed

from anchorpy import Program, Provider, Wallet
from spl.token.async_client import AsyncToken
from spl.token.constants import TOKEN_PROGRAM_ID

from switchboardpy.aggregator import AggregatorAccount
from google.protobuf.internal import encoder

class SwitchboardProgram(object):

    async def __aenter__(self):
        client = AsyncClient("https://api.devnet.solana.com/", commitment=Confirmed)
        provider = Provider(client, Wallet.local())
        self.program = await Program.at(
            SBV2_DEVNET_PID, provider
        )
        return self.program
    
    async def __aexit__(self, exc_t, exc_v, exc_tb):
        await self.program.close()

@mark.asyncio
async def test_create():
    async with SwitchboardProgram() as program:

        # Get Permissionless Queue Devnet
        queue = OracleQueueAccount(
            AccountParams(
                program=program, 
                public_key=PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy")
            )
        )

        # Create aggregator so we can later make a lease for it
        aggregator = await AggregatorAccount.create(
            program=program, 
            aggregator_init_params=AggregatorInitParams(
                batch_size=3, 
                min_required_oracle_results=2, 
                min_required_job_results=1, 
                min_update_delay_seconds=6, 
                queue_account=OracleQueueAccount(
                    AccountParams(
                        program=program, 
                        public_key=PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy")
                    )
                ),
                start_after=0,
            )
        )

        # Create Job Definition
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
        delimiter = encoder._VarintBytes(len(serializedMessage)) # Rust Crate Requires Encode Delimited Proto
        delimitedOJ = delimiter + serializedMessage

        # Create Job on Chain
        job = await JobAccount.create(
            program=program, 
            params=JobInitParams(
              data=delimitedOJ
            )
        )

        # Add SOL / USD job to Aggregator
        await aggregator.add_job(job)

        queue_data = await queue.load_data()

        # Create Permission Account
        await PermissionAccount.create(
            program,
            PermissionInitParams(
                granter=queue.public_key,
                grantee=aggregator.public_key,
                authority=queue_data.authority
            )
        )

        # Create tokenAccount to fund lease
        tokenAccount = await AsyncToken.create_wrapped_native_account(
            program.provider.connection, 
            TOKEN_PROGRAM_ID, 
            program.provider.wallet.public_key, 
            program.provider.wallet.payer, 
            1_000_000,
            skip_confirmation=False
        )

        # Create lease
        lease = await LeaseAccount.create(
            program=program, 
            params=LeaseInitParams(
                withdraw_authority=program.provider.wallet.public_key,
                load_amount=1_000_000, 
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
        )

        # Get crank
        crank = CrankAccount(AccountParams(program=program, public_key=PublicKey("GN9jjCy2THzZxhYqZETmPM3my8vg4R5JyNkgULddUMa5")))
        
        # Add Aggregator for auto-updates
        await crank.push(CrankPushParams(aggregator_account=aggregator))


        print(f'Feed info at: https://switchboard.xyz/explorer/2/{aggregator.public_key}')