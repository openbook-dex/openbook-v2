# switchboardv2-py-api

---

SwitchboardPy is the Python client for [Switchboard](https://docs.switchboard.xyz/introduction). It provides wrappers to help you to interact with the Switchboard V2 program on-chain.

Internally it uses [AnchorPy](https://kevinheavey.github.io/anchorpy/), an Anchor API implementation in Python.

[![pypi](https://badgen.net/pypi/v/switchboardpy)](https://pypi.python.org/pypi/switchboardpy)&nbsp;&nbsp;
[![twitter](https://badgen.net/twitter/follow/switchboardxyz)](https://twitter.com/switchboardxyz)&nbsp;&nbsp;

## Installation

```sh
pip install switchboardpy
```

## Basic Usage

```python
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

    # getting most recent value (decimal.Decimal)
    val = await agg.get_latest_value()

    print('LATEST VALUE:')
    print(val)

    await program.close()

asyncio.run(main())

"""
OUTPUT
LATEST VALUE:
180.12115
"""

```

## Anchorpy Client Gen

```sh
anchorpy client-gen ./switchboard_v2.mainnet.parsed.json ./switchboardpy --program-id SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f
```
