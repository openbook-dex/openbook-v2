import asyncio

from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.rpc.async_api import AsyncClient
from anchorpy import Program, Provider, Wallet
from switchboardpy import AggregatorAccount, AccountParams

# Devnet Program ID.
SBV2_DEVNET_PID = PublicKey(
    '2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG'
)

async def main():
    client = AsyncClient("https://api.devnet.solana.com/")
    provider = Provider(client, Wallet(Keypair()))
    program = await Program.at(
        SBV2_DEVNET_PID, provider
    )
    agg = AggregatorAccount(AccountParams(program=program, public_key=PublicKey("88FX4tBstuwBPNhQU4EEBoPX35neSu4Le9zDSwtPRRQz")))

    # getting aggregator data
    data = await agg.load_data()

    # getting most recent value
    val = await agg.get_latest_value()

    print('LATEST VALUE:')
    print(val)
    print(program.type['SwitchboardDecimal'](1, 3))

    await program.close()

asyncio.run(main())


