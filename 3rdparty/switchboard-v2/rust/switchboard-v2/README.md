[![cargo](https://badgen.net/crates/v/switchboard-v2)](https://crates.io/crates/switchboard-v2)&nbsp;&nbsp;
[![twitter](https://badgen.net/twitter/follow/switchboardxyz)](https://twitter.com/switchboardxyz)&nbsp;&nbsp;

# switchboard-v2

A Rust library to interact with Switchboard V2 accounts.

This package can be used to interact and deserialize Switchboard V2 accounts.

See the [documentation](https://docs.switchboard.xyz/introduction) for more info
on Switchboard.

## Features

| Feature | Description                                                                                           |
| ------- | ----------------------------------------------------------------------------------------------------- |
| devnet  | The devnet feature enables using the Switchboard Devnet Program ID instead of the Mainnet Program ID. |

Enable it in your Cargo.toml

```toml
[dependencies]
switchboard-v2 = { version = "0.1.17", features = ["devnet"] }
```

### Define Your Own Devnet Feature

You can also define your own devnet feature to dynamically swap the program IDs.

```toml
[features]
default = []
devnet = ["switchboard-v2/devnet"]
```

This allows you to build your program with a feature flag to automate devnet and
mainnet builds.

```bash
# Build with Mainnet Switchboard Program ID
cargo build-bpf
# Build with Devnet Switchboard Program ID
cargo build-bpf --features devnet
```

## Usage

### Aggregator

#### Read Latest Result

```rust
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, SWITCHBOARD_PROGRAM_ID};

// check feed owner
let owner = *aggregator.owner;
if owner != SWITCHBOARD_PROGRAM_ID {
    return Err(error!(ErrorCode::InvalidSwitchboardAccount));
}

// deserialize account info
let feed = ctx.accounts.aggregator.load()?;
// OR
let feed = AggregatorAccountData::new(feed_account_info)?;

// get result
let decimal: f64 = feed.get_result()?.try_into()?;

// check if feed has been updated in the last 5 minutes
feed.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;

// check if feed exceeds a confidence interval of +/i $0.80
feed.check_confidence_interval(SwitchboardDecimal::from_f64(0.80))?;
```

**Example(s)**:
[anchor-feed-parser](https://github.com/switchboard-xyz/sbv2-solana/blob/main/examples/programs/anchor-feed-parser/src/lib.rs),
[native-feed-parser](https://github.com/switchboard-xyz/sbv2-solana/blob/main/examples/programs/native-feed-parser/src/lib.rs)

#### Read Aggregator History

**_Note: The Aggregator must have a history buffer initialized before using_**

```rust
use switchboard_v2::AggregatorHistoryBuffer;
use std::convert::TryInto;

let history_buffer = AggregatorHistoryBuffer::new(history_account_info)?;
let current_timestamp = Clock::get()?.unix_timestamp;
let one_hour_ago: f64 = history_buffer.lower_bound(current_timestamp - 3600).unwrap().try_into()?;
```

### VRF Account

#### Read Latest Result

```rust
use switchboard_v2::VrfAccountData;

// deserialize the account info
let vrf = ctx.accounts.vrf.load()?;
// OR
let vrf = VrfAccountData::new(vrf_account_info)?;

// read the result
let result_buffer = vrf.get_result()?;
let value: &[u128] = bytemuck::cast_slice(&result_buffer[..]);
let result = value[0] % 256000 as u128;
```

**Example**:
[anchor-vrf-parser](https://github.com/switchboard-xyz/sbv2-solana/blob/main/examples/programs/anchor-vrf-parser/src/actions/update_result.rs)

#### RequestRandomness CPI

```rust
pub use switchboard_v2::{VrfAccountData, VrfRequestRandomness};

let switchboard_program = ctx.accounts.switchboard_program.to_account_info();

let vrf_request_randomness = VrfRequestRandomness {
    authority: ctx.accounts.state.to_account_info(),
    vrf: ctx.accounts.vrf.to_account_info(),
    oracle_queue: ctx.accounts.oracle_queue.to_account_info(),
    queue_authority: ctx.accounts.queue_authority.to_account_info(),
    data_buffer: ctx.accounts.data_buffer.to_account_info(),
    permission: ctx.accounts.permission.to_account_info(),
    escrow: ctx.accounts.escrow.clone(),
    payer_wallet: ctx.accounts.payer_wallet.clone(),
    payer_authority: ctx.accounts.payer_authority.to_account_info(),
    recent_blockhashes: ctx.accounts.recent_blockhashes.to_account_info(),
    program_state: ctx.accounts.program_state.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
};

let vrf_key = ctx.accounts.vrf.key.clone();
let authority_key = ctx.accounts.authority.key.clone();

let state_seeds: &[&[&[u8]]] = &[&[
    &STATE_SEED,
    vrf_key.as_ref(),
    authority_key.as_ref(),
    &[bump],
]];
msg!("requesting randomness");
vrf_request_randomness.invoke_signed(
    switchboard_program,
    params.switchboard_state_bump,
    params.permission_bump,
    state_seeds,
)?;

```

**Example**:
[anchor-vrf-parser](https://github.com/switchboard-xyz/sbv2-solana/blob/main/examples/programs/anchor-vrf-parser/src/actions/request_result.rs)

### Buffer Relayer Account

#### Read Latest Result

```rust
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
use switchboard_v2::{BufferRelayerAccountData, SWITCHBOARD_PROGRAM_ID};

// check feed owner
let owner = *aggregator.owner;
if owner != SWITCHBOARD_PROGRAM_ID {
    return Err(error!(ErrorCode::InvalidSwitchboardAccount));
}

// deserialize account info
let buffer = BufferRelayerAccountData::new(feed_account_info)?;

// get result
let buffer_result = buffer.get_result();

// check if feed has been updated in the last 5 minutes
buffer.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;

// convert buffer to a string
let result_string = String::from_utf8(buffer.result)
    .map_err(|_| error!(ErrorCode::StringConversionFailed))?;
msg!("Buffer string {:?}!", result_string);
```

**Example**:
[anchor-buffer-parser](https://github.com/switchboard-xyz/sbv2-solana/blob/main/examples/programs/anchor-buffer-parser/src/lib.rs)
