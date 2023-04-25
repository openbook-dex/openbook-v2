# Switchboard-V2 Feed Walkthrough

This example will walk you through

- creating a personal oracle queue with a crank
- add a SOL/USD data feed onto the crank
- spin up a docker environment to run your own oracle
- fulfill your update request on-chain

## Usage

```bash
ts-node src/main [PAYER_KEYPAIR_PATH]
```

where **PAYER_KEYPAIR_PATH** is the location of your Solana keypair, defaulting
to `~/.config/solana/id.json` if not provided

When prompted, run the docker compose script in a new shell to start your local
oracle then confirm the prompt to turn the crank and request an update on-chain.
The oracle is ready to fulfill updates when it sees the following logs:

```bash
{"timestamp":"2022-09-23T19:24:11.874Z","level":"info","message":"Loaded 1000 nonce accounts"}
{"timestamp":"2022-09-23T19:24:11.885Z","level":"info","message":"started health check handler"}
{"timestamp":"2022-09-23T19:24:11.886Z","level":"info","message":"Heartbeat routine started with an interval of 15 seconds."}
{"timestamp":"2022-09-23T19:24:11.887Z","level":"info","message":"Watching event: AggregatorOpenRoundEvent ..."}
{"timestamp":"2022-09-23T19:24:11.893Z","level":"info","message":"Watching event: VrfRequestRandomnessEvent ..."}
{"timestamp":"2022-09-23T19:24:11.894Z","level":"info","message":"Using default performance monitoring"}
```

Example Output:

```bash
$ ts-node src/main
######## Switchboard Setup ########
Program State            BYM81n8HvTJuqZU1PmTVcwZ9G8uoji7FKM6EaPkwphPt
Oracle Queue             AVbBmSeKJppRcphaPY1fPbFQW48Eg851G4XfqyTPMZNF
Crank                    6fNsrJhaB2MPpwpcxW7AL5zyoiq7Gyz2mM6q3aVz7xxh
Oracle                   CmTr9FSeuhMPBLEPa3o2M71RwRnBz6LMcsfzHaW721Ak
  Permission             2pC5ESkVKGx4yowGrVB21f6eXaaMRQY5cBazfqn1bAQs
Aggregator (SOL/USD)     FLixyyJVzfCF4PmDG2VcFm1LUBu1aBTXox3oCWNVU88m
  Permission             EVerqanwRrHRvtPXDRdFHPc7VnXuyEPRr9XA5udpFA4E
  Lease                  FC6SfAEuoB1SoZAnCqkMyyYnSfLSy8KfPUFH9SASBUzU
  Job (FTX)              BbNzfRQjTYiCZVfvK1qpQkkon3kP2tbvaCHfzsyjeBU3
✔ Switchboard setup complete
######## Start the Oracle ########
Run the following command in a new shell

      ORACLE_KEY=CmTr9FSeuhMPBLEPa3o2M71RwRnBz6LMcsfzHaW721Ak PAYER_KEYPAIR=/Users/switchboard/.config/solana/id.json RPC_URL=https://api.devnet.solana.com docker-compose up

Select 'Y' when the docker container displays Starting listener... [y/n]: y

✔ Crank turned
######## Aggregator Result ########
Result: 30.91

✔ Aggregator succesfully updated!
```
