from __future__ import annotations

import struct
import anchorpy
import time
import hashlib

from dataclasses import dataclass
from decimal import Decimal
from typing import Optional, Any, NamedTuple

from solana.keypair import Keypair
from solana.publickey import PublicKey
from solana.transaction import TransactionSignature
from solana.rpc.commitment import Confirmed
from solana.system_program import CreateAccountParams, create_account

from spl.token.async_client import AsyncToken
from spl.token.constants import TOKEN_PROGRAM_ID
from spl.token.instructions import get_associated_token_address

from switchboardpy.compiled import OracleJob
from switchboardpy.common import AccountParams, SwitchboardDecimal, parseOracleJob

from switchboardpy.program import ProgramStateAccount
from switchboardpy.oraclequeue import OracleQueueAccount
from switchboardpy.oracle import OracleAccount
from switchboardpy.job import JobAccount
from switchboardpy.lease import LeaseAccount
from switchboardpy.permission import PermissionAccount

from .generated.accounts import AggregatorAccountData

# Parameters for which oracles must submit for responding to update requests.
@dataclass
class AggregatorSaveResultParams:

    """Index in the list of oracles in the aggregator assigned to this round update."""
    oracle_idx: int

    """Reports that an error occured and the oracle could not send a value."""
    error: bool

    """Value the oracle is responding with for this update."""
    value: Decimal

    """
    The minimum value this oracle has seen this round for the jobs listed in the
    aggregator.
    """
    min_response: Decimal

    """
    The maximum value this oracle has seen this round for the jobs listed in the
    aggregator.
    """
    max_response: Decimal

    """List of OracleJobs that were performed to produce this result"""
    jobs: list[OracleJob]

    """Authority of the queue the aggregator is attached to"""
    queue_authority: PublicKey

    """Program token mint"""
    token_mint: PublicKey

    """List of parsed oracles"""
    oracles: list[Any]


# Parameters for creating and setting a history buffer
@dataclass
class AggregatorSetHistoryBufferParams:
    
    """Number of elements for the history buffer to fit"""
    size: int

    """Authority keypair for the aggregator"""
    authority: Keypair = None

# Parameters for creating and setting a history buffer
@dataclass
class AggregatorSetUpdateIntervalParams:
    
    """Seconds between updates"""
    new_interval: int

    """Authority keypair for the aggregator"""
    authority: Keypair = None

# Parameters required to open an aggregator round
@dataclass
class AggregatorOpenRoundParams:

    """The oracle queue from which oracles are assigned this update."""
    oracle_queue_account: OracleQueueAccount
    
    """The token wallet which will receive rewards for calling update on this feed."""
    payout_wallet: PublicKey

    """
    Data feeds on a crank are ordered by their next available update time with some 
    level of jitter to mitigate oracles being assigned to the same update request upon 
    each iteration of the queue, which makes them susceptible to a malicous oracle. 
    """
    jitter: int = None

# Result of returning loadedJobs
@dataclass
class AggregatorLoadedJob:

    """The oracle queue from which oracles are assigned this update."""
    job: OracleJob
    
    """Public Key of a given job"""
    public_key: PublicKey

    """Job account data"""
    account: Any



# Parameters required to set min jobs for Aggregator
@dataclass
class AggregatorSetMinJobsParams:

    """The min number of jobs required"""
    min_job_results: int
    
    """The feed authority."""
    authority: Keypair = None


# Parameters required to set batch size for Aggregator
@dataclass
class AggregatorSetBatchSizeParams:

    """The batch size."""
    batch_size: int
    
    """The feed authority."""
    authority: Keypair = None


# Parameters required to set min oracles for Aggregator
@dataclass
class AggregatorSetMinOraclesParams:

    """The min results required"""
    min_oracle_results: int
    
    """The feed authority."""
    authority: Keypair = None


# Parameters required to set min oracles for Aggregator
@dataclass
class AggregatorSetQueueParams:

    """The min results required"""
    queue_account: OracleQueueAccount
    
    """The feed authority."""
    authority: Keypair = None

# Parameters required to set min oracles for Aggregator
@dataclass
class AggregatorSetVarianceThresholdParams:

    """The % change needed to trigger an update"""
    threshold: Decimal
    
    """The feed authority."""
    authority: Keypair = None

# Init Params for Aggregators
@dataclass
class AggregatorInitParams:
    """Number of oracles to request on aggregator update."""
    batch_size: int

    """Minimum number of oracle responses required before a round is validated."""
    min_required_oracle_results: int

    """Minimum number of seconds required between aggregator rounds."""
    min_required_job_results: int

    """Minimum number of seconds required between aggregator rounds."""
    min_update_delay_seconds: int

    """The queue to which this aggregator will be linked"""
    queue_account: OracleQueueAccount
 
    """Name of the aggregator to store on-chain."""
    name: bytes = None

    """Metadata of the aggregator to store on-chain."""
    metadata: bytes = None

    """unix_timestamp for which no feed update will occur before."""
    start_after: int = None

    """
    Change percentage required between a previous round and the current round.
    If variance percentage is not met, reject new oracle responses.
    """
    variance_threshold: Decimal = None

    """
    Number of seconds for which, even if the variance threshold is not passed,
    accept new responses from oracles.
    """
    force_report_period: int = None

    """
    unix_timestamp after which funds may be withdrawn from the aggregator.
    null/undefined/0 means the feed has no expiration.
    """
    expiration: int = None

    """
    An optional wallet for receiving kickbacks from job usage in feeds.
    Defaults to token vault.
    """
    keypair: Keypair = None
    
    """
    An optional wallet for receiving kickbacks from job usage in feeds.
    Defaults to token vault.
    """
    author_wallet: PublicKey = None

    """
    If included, this keypair will be the aggregator authority rather than
    the aggregator keypair.
    """
    authority: PublicKey = None

    """Disable automatic updates"""
    disable_crank: bool = None


@dataclass
class AggregatorHistoryRow:
    """AggregatorHistoryRow is a wrapper for the row structure of elements in the aggregator history buffer.
    
    Attributes:
        timestamp (int): timestamp of the aggregator result
        value (Decimal): Aggregator value at the timestamp
    """
    timestamp: int
    value: Decimal
    
    """
    Generate an AggregatorHistoryRow from a retrieved buffer representation

    Args:
        buf (list): Anchor-loaded buffer representation of AggregatorHistoryRow

    Returns:
        AggregatorHistoryRow
    """
    @staticmethod
    def from_buffer(buf: bytes):
        timestamp: int = struct.unpack_from("<L", buf[:8])[0]
        mantissa: int = struct.unpack_from("<L", buf[8:24])[0]
        scale: int = struct.unpack_from("<L", buf, 24)[0]
        decimal = SwitchboardDecimal.sbd_to_decimal({"mantissa": mantissa, "scale": scale})
        res = AggregatorHistoryRow(timestamp, decimal)
        return res


class AggregatorAccount:
    """AggregatorAccount is the wrapper for an Aggregator, the structure for that keeps aggregated feed data / metadata.

    Attributes:
        program (anchor.Program): The anchor program ref
        public_key (PublicKey | None): This aggregator's public key
        keypair (Keypair | None): this aggregator's keypair
    """


    def __init__(self, params: AccountParams):
        if params.public_key is None and params.keypair is None:
            raise ValueError('User must provide either a publicKey or keypair for account use.')
        if params.keypair and params.public_key and params.keypair.public_key != params.public_key:
            raise ValueError('User must provide either a publicKey or keypair for account use.')
        self.program = params.program
        self.public_key = params.keypair.public_key if params.keypair else params.public_key
        self.keypair = params.keypair
    

    """
    Get name of an aggregator.

    Args:
        aggregator (Any): Anchor-loaded aggregator

    Returns:
        name string of the aggregator
    """
    @staticmethod
    def get_name(aggregator: Any) -> str:
        return  ''.join(map(chr, *aggregator.name)).decode("utf-8").replace(u"\u0000", "*").encode("utf-8")

        
    """
    Load and parse AggregatorAccount state based on the program IDL. 
    
    Returns:
        name (AggregatorAccount): data parsed in accordance with the
            Switchboard IDL.

    Args:

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_data(self):
        return await AggregatorAccountData.fetch(self.program.provider.connection, self.public_key)
        

    """
    Get AggregatorAccount historical data 

    Returns:
        name (AggregatorAccount): data parsed in accordance with the
            Switchboard IDL.

    Args:
        aggregator (Any): Optional aggregator 

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_history(self, aggregator: Any = None) -> Any:

        # if aggregator data passed in - use that, else load this aggregator
        aggregator = aggregator if aggregator else await self.load_data()

        # Compare History Buffer to default public key (zeroed out)
        if (aggregator.history_buffer == 11111111111111111111111111111111):
            return []

        # Fixed AggregatorHistoryRow size
        ROW_SIZE = 28

        # Get account data
        info = await self.program.provider.connection.get_account_info(aggregator.history_buffer) 
        buffer = info.data if info else []
        if not buffer or buffer.length < 12:
            return []
        
        # Read UInt32 as a Little Endian val, starting at position 8
        insert_idx: int = struct.unpack_from("<L", buffer, 8)[0] * ROW_SIZE

        front = []
        tail = []

        if not isinstance(buffer, list):
            return []
        
        for i in range(13, buffer.length, ROW_SIZE):
            if i + ROW_SIZE > buffer.length:
                break
            row = AggregatorHistoryRow.from_buffer(buffer)
            if row.timestamp == 0:
                break
            if i <= insert_idx:
                tail.append(row)
            else:
                front.append(row)
        return front.extend(tail)

    """
    Get the latest confirmed value stored in the aggregator account. 
    
    Args:
        aggregator (Any): Optional aggregator value to pass in

    Returns:
        value (Decimal): the latest feed value

    Raises:
        ValueError: If the aggregator currently holds no value
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def get_latest_value(self, aggregator: Optional[Any] = None) -> Decimal:
        aggregator = aggregator if aggregator else await self.load_data()
        if hasattr(aggregator, 'latest_confirmed_round') and aggregator.latest_confirmed_round.num_success == 0:
            raise ValueError('Aggregator currently holds no value.')
        return SwitchboardDecimal.sbd_to_decimal(aggregator.latest_confirmed_round.result)


    """
    Get the timestamp latest confirmed round stored in the aggregator account. 
    
    Args:
        aggregator (Any): Optional aggregator value to pass in

    Returns:
        timestamp (str): latest feed timestamp as hex string

    Raises:
        ValueError: If the aggregator currently holds no value
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def get_latest_feed_timestamp(self, aggregator: Optional[Any] = None) -> Decimal:
        aggregator = aggregator if aggregator else await self.load_data()
        if hasattr(aggregator, 'latest_confirmed_round') and aggregator.latest_confirmed_round.num_success == 0:
            raise ValueError('Aggregator currently holds no value.')

        return aggregator.latest_confirmed_round.round_open_timestamp


    """
    Get name of an aggregator.

    Args:
        aggregator (any): Anchor-loaded aggregator

    Returns:
        name string of the aggregator
    """
    @staticmethod
    def should_report_value(value: Decimal, aggregator: Optional[Any] = None) -> bool:
        if aggregator.latestConfirmedRound and aggregator.latest_confirmed_round.num_success == 0:
            return True
        timestamp = round(int(time.time()) / 1000)
        if aggregator.start_after > timestamp:
            return False
        variance_threshold = SwitchboardDecimal.sbd_to_decimal(aggregator.variance_threshold)
        latest_result = SwitchboardDecimal.sbd_to_decimal(aggregator.latest_confirmed_round.result)
        force_report_period = aggregator.force_report_period
        last_timestamp = aggregator.latest_confirmed_round.round_open_timestamp
        if last_timestamp + force_report_period < timestamp:
            return True
        diff = latest_result / value
        if abs(diff) > 1:
            diff = value / latest_result
        if diff < 0:
            return True
        
        change_percentage = 1 - diff * 100
        return change_percentage > variance_threshold

    """
    Get the individual oracle results of the latest confirmed round. 
    
    Args:
        aggregator (Any): Optional aggregator value to pass in

    Returns:
        timestamp (str): latest feed timestamp as hex string

    Raises:
        ValueError: If aggregator currently holds no value.
    """
    async def get_confirmed_round_results(self, aggregator: Optional[Any] = None) -> Decimal:
        
        aggregator = aggregator if aggregator else await self.load_data()
        if hasattr(aggregator, 'latest_confirmed_round') and aggregator.latest_confirmed_round.num_success == 0:
            raise ValueError('Aggregator currently holds no value.')
        results: list[Any] = []
        for i in range(aggregator.oracle_request_batch_size):
            if aggregator.latest_confirmed_round.medians_filfilled[i]:
                results.append({
                    "oracle_account": OracleAccount(AccountParams(program=self.program, public_key=aggregator.latest_confirmed_round.oracle_pubkeys_data[i])),
                    "value": SwitchboardDecimal.sbd_to_decimal(aggregator.latest_confirmed_round.medians_data[i])
                })
        return results

    """
    Get the hash of a list of OracleJobs
    
    Args:
        jobs (list[OracleJob]): list of jobs to hash

    Returns:
        hash (_Hash): hash as hex string

    Raises:
    """
    @staticmethod
    def produce_job_hash(jobs: list[OracleJob]):
        hash = hashlib.sha256()
        for job in jobs:
            job_hasher = hashlib.sha256()
            job_hasher.update(job.SerializeToString())
            hash.update(job_hasher.digest())
        return hash


    """
    Load and deserialize all jobs stored in this aggregator
    
    Args:
        aggregator (Any): Optional aggregator

    Returns:
        jobs (list[{ "job": OracleJob, "public_key": PublicKey, "account": JobAccountData }]) 

    Raises:
        ValueError: Failed to load feed jobs.
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_jobs(self, aggregator: Optional[Any] = None) -> Decimal:
        coder = anchorpy.AccountsCoder(self.program.idl)
        aggregator = aggregator if aggregator else await self.load_data()
        job_accounts_raw = await anchorpy.utils.rpc.get_multiple_accounts(self.program.provider.connection, aggregator.job_pubkeys_data[:aggregator.job_pubkeys_size], 10, Confirmed)
        if not job_accounts_raw:
            raise ValueError('Failed to load feed jobs.')
        
        # Deserialize OracleJob objects from each decoded JobAccountData 
        return [AggregatorLoadedJob(parseOracleJob(coder.decode(job.account.data).data), job.pubkey, coder.decode(job.account.data)) for job in job_accounts_raw]
        
    """
    Load all job hashes for each job stored in this aggregator
    
    Args:
        aggregator (Any): Optional aggregator

    Returns:
        hashes (list[str]): hashes for each job 

    Raises:
        AccountDoesNotExistError: If the account doesn't exist.
        AccountInvalidDiscriminator: If the discriminator doesn't match the IDL.
    """
    async def load_hashes(self, aggregator: Optional[Any] = None) -> Decimal:
        coder = anchorpy.AccountsCoder(self.program.idl)
        aggregator = aggregator if aggregator else await self.loadData()
        job_accounts_raw = await anchorpy.utils.rpc.get_multiple_accounts(self.program.provider.connection, aggregator.job_pubkeys_data[:aggregator.job_pubkeys_size])
        if not job_accounts_raw:
            raise ValueError('Failed to load feed jobs.')
        
        # get hashes from each decoded JobAccountData 
        return [coder.decode(job.account.data).hash for job in job_accounts_raw]
        
    
    """
    Get the size of an AggregatorAccount on chain
    
    Returns:
        int: size of the AggregatorAccount on chain
    """
    def size(self):
        return self.program.account["AggregatorAccountData"].size

    """
    Create and initialize the AggregatorAccount.
    
    Args:
        program (anchorpy.Program): Switchboard program representation holding connection and IDL
        params (AggregatorInitParams): init params for the aggregator

    Returns:
        AggregatorAccount
    """
    @staticmethod
    async def create(program: anchorpy.Program, aggregator_init_params: AggregatorInitParams):
        aggregator_account = aggregator_init_params.keypair or Keypair.generate()
        authority = aggregator_init_params.authority or aggregator_account.public_key
        size = program.account["AggregatorAccountData"].size
        state_account, state_bump = ProgramStateAccount.from_seed(program)
        state = await state_account.load_data()
        response = await program.provider.connection.get_minimum_balance_for_rent_exemption(size)
        lamports = response["result"]
        zero_decimal = SwitchboardDecimal(0, 0).as_proper_sbd(program)

        await program.rpc["aggregator_init"](
            {
                "name": aggregator_init_params.name or bytes([0] * 32),
                "metadata": aggregator_init_params.metadata or bytes([0] * 128),
                "batch_size": aggregator_init_params.batch_size,
                "min_oracle_results": aggregator_init_params.min_required_oracle_results,
                "min_job_results": aggregator_init_params.min_required_job_results,
                "min_update_delay_seconds": aggregator_init_params.min_update_delay_seconds,
                "variance_threshold": SwitchboardDecimal.from_decimal(aggregator_init_params.variance_threshold).as_proper_sbd(program) if aggregator_init_params.variance_threshold else zero_decimal,
                "force_report_period": aggregator_init_params.force_report_period or 0,
                "expiration": aggregator_init_params.expiration or 0,
                "state_bump": state_bump,
                "disable_crank": aggregator_init_params.disable_crank or False,
                "start_after": aggregator_init_params.start_after or 0,
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": aggregator_account.public_key,
                    "authority": authority,
                    "queue": aggregator_init_params.queue_account.public_key,
                    "author_wallet": aggregator_init_params.author_wallet or state.token_vault,
                    "program_state": state_account.public_key
                },
                signers=[aggregator_account],
                pre_instructions=[
                    create_account(
                        CreateAccountParams(
                            from_pubkey=program.provider.wallet.public_key, 
                            new_account_pubkey=aggregator_account.public_key,
                            lamports=lamports, 
                            space=size, 
                            program_id=program.program_id
                        )
                    )
                ]
            )
        )
        return AggregatorAccount(AccountParams(program=program, keypair=aggregator_account))

    """
    Create and set a history buffer for the aggregator

    Args:
        program (anchorpy.Program): Switchboard program representation holding connection and IDL
        params (AggregatorSetHistoryBufferParams)

    Returns:
        TransactionSignature
    """
    async def set_history_buffer(self, params: AggregatorSetHistoryBufferParams):
        buffer = Keypair.generate()
        program = self.program
        authority = params.authority or self.keypair
        HISTORY_ROW_SIZE = 28
        INSERT_IDX_SIZE = 4
        DISCRIMINATOR_SIZE = 8
        size = params.size * HISTORY_ROW_SIZE + INSERT_IDX_SIZE + DISCRIMINATOR_SIZE
        response = await program.provider.connection.get_minimum_balance_for_rent_exemption(size)
        lamports = response["result"]
        await program.rpc["aggregator_set_history_buffer"](
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                    "buffer": buffer.public_key
                },
                signers=[authority, buffer],
                pre_instructions=[
                    create_account(
                        CreateAccountParams(
                            from_pubkey=program.provider.wallet.public_key,
                            new_account_pubkey=buffer.public_key,
                            space=size,
                            lamports=lamports,
                            program_id=program.program_id
                        )
                    )
                ]
            )
        )

    """
    Open round on aggregator to get an update

    Args:
        program (anchorpy.Program): Switchboard program representation holding connection and IDL
        params (AggregatorOpenRoundParams)

    Returns:
        TransactionSignature
    """
    async def open_round(self, params: AggregatorOpenRoundParams):
        program = self.program
        state_account, state_bump = ProgramStateAccount.from_seed(program)
        queue = await params.oracle_queue_account.load_data()
        lease_account, lease_bump = LeaseAccount.from_seed(
            self.program,
            params.oracle_queue_account,
            self
        )
        lease = await lease_account.load_data()
        permission_account, permission_bump = PermissionAccount.from_seed(
            self.program,
            queue.authority,
            params.oracle_queue_account.public_key,
            self.public_key
        )
        return await program.rpc["aggregator_open_round"](
            {
                "state_bump": state_bump,
                "lease_bump": lease_bump,
                "permission_bump": permission_bump,
                "jitter": params.jitter or 0
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "lease":  lease_account.public_key,
                    "oracle_queue": params.oracle_queue_account.public_key,
                    "queue_authority": queue.authority,
                    "permission": permission_account.public_key,
                    "escrow": lease.escrow,
                    "program_state": state_account.public_key,
                    "payout_wallet": params.payout_wallet,
                    "token_program": TOKEN_PROGRAM_ID,
                    "data_buffer": queue.data_buffer,
                    "mint": (await params.oracle_queue_account.load_mint()).pubkey,
                },
            )
        )

    """
    Set min jobs sets the min jobs parameter. This is a suggestion to oracles 
    of the number of jobs that must resolve for a job to be considered valid.

    Args:
        params (AggregatorSetMinJobsParams): parameters pecifying the min jobs that must respond
    Returns:
        TransactionSignature

    """
    async def set_min_jobs(self, params: AggregatorSetMinJobsParams):
        authority = authority or self.keypair
        return await self.program.rpc['aggregator_set_min_jobs'](
            {
                "min_job_results": params.min_job_results
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )

    """
    Set min oracles sets the min oracles parameter. This will determine how many oracles need to come back with a 
    valid response for a result to be accepted. 

    Args:
        params (AggregatorSetMinOraclesParams): parameters pecifying the min jobs that must respond
    Returns:
        TransactionSignature

    """
    async def set_min_jobs(self, params: AggregatorSetMinJobsParams):
        authority = authority or self.keypair
        return await self.program.rpc['aggregator_set_min_jobs'](
            {
                "min_job_results": params.min_job_results
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )
    
    """
    RPC to add a new job to an aggregtor to be performed on feed updates.

    Args:
        job (JobAccount): specifying another job for this aggregator to fulfill on update
        authority (Keypair | None)
    Returns:
        TransactionSignature
    """
    async def add_job(self, job: JobAccount, weight: int = 0, authority: Optional[Keypair] = None) -> TransactionSignature:
        authority = authority or self.keypair
        
        return await self.program.rpc['aggregator_add_job'](
            {
                "weight": weight
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                    "job": job.public_key
                },
                signers=[authority]
            )
        )

    """
    RPC Set batch size / the number of oracles that'll respond to updates

    Args:
        params (AggregatorSetBatchSizeParams)
    Returns:
        TransactionSignature
    """
    async def set_batch_size(self, params: AggregatorSetBatchSizeParams) -> TransactionSignature:
        authority = authority or self.keypair
        return await self.program.rpc['aggregator_set_batch_size'](
            {
                "batch_size": params.batch_size
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )
        

    """
    RPC set variance threshold (only write updates when response is > variance threshold %)

    Args:
        params (AggregatorSetVarianceThresholdParams)
    Returns:
        TransactionSignature
    """
    async def set_variance_threshold(self, params: AggregatorSetVarianceThresholdParams) -> TransactionSignature:
        authority = authority or self.keypair
        
        return await self.program.rpc['aggregator_set_variance_threshold'](
            {
                "variance_threshold": SwitchboardDecimal.from_decimal(params.threshold)
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )

    """
    RPC set min oracles

    Args:
        params (AggregatorSetMinOraclesParams)
    Returns:
        TransactionSignature
    """
    async def set_min_oracles(self, params: AggregatorSetMinOraclesParams) -> TransactionSignature:
        authority = authority or self.keypair
        
        return await self.program.rpc['aggregator_set_min_oracles'](
            {
                "min_oracle_results": params.min_oracle_results
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )

    """
    RPC set update interval

    Args:
        params (AggregatorSetUpdateIntervalParams)
    Returns:
        TransactionSignature
    """
    async def set_update_interval(self, params: AggregatorSetUpdateIntervalParams) -> TransactionSignature:
        authority = authority or self.keypair
        
        return await self.program.rpc['aggregator_set_update_interval'](
            {
                "new_interval": params.new_interval
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )
        
    """
    Prevent new jobs from being added to the feed.

    Args:
        authority (Keypair | None): the current authority keypair

    Returns:
        TransactionSignature
    """
    async def lock(self, authority: Optional[Keypair] = None) -> TransactionSignature:
        authority = authority or self.keypair
        return await self.program.rpc['aggregator_lock'](
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                },
                signers=[authority]
            )
        )

    """
    Change the aggregator authority

    Args:
        new_authority (Keypair): The new authority
        current_authority (Keypair | None): the current authority keypair

    Returns:
        TransactionSignature
    """
    async def set_authority(self, new_authority: Keypair, current_authority: Optional[Keypair] = None) -> TransactionSignature:
        current_authority = current_authority or self.keypair
        return await self.program.rpc['aggregator_set_authoirty'](
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "new_authority": new_authority,
                    "authority": current_authority.public_key,
                },
                signers=[current_authority]
            )
        )

    """
    RPC to add remove job from an aggregtor.

    Args:
        job (JobAccount): specifying job to remove
        authority (Keypair | None)
    Returns:
        TransactionSignature
    """
    async def remove_job(self, job: JobAccount, authority: Optional[Keypair] = None) -> TransactionSignature:
        authority = authority or self.keypair
        return await self.program.rpc['aggregator_remove_job'](
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "authority": authority.public_key,
                    "job": job.public_key
                },
                signers=[authority]
            )
        )
    
    """
    Get Index of Oracle in Aggregator

    Args:
        oracle_pubkey (PublicKey): Public key belonging to the oracle

    Returns:
        int: index of the oracle, -1 if not found
    """
    async def get_oracle_index(self, oracle_pubkey: PublicKey):
        aggregator = await self.load_data()
        for i, curr_oracle_pubkey in enumerate(aggregator.current_round.oracle_pubkeys_data):
            if curr_oracle_pubkey == oracle_pubkey:
                return i
        return -1
    
    """
    Save Aggregator result

    Args:
        aggregator (Any): Aggregator data
        oracle_account (OracleAccount)
        params (AggregatorSaveResultParams)

    Returns:
        TransactionSignature
    """
    async def remove_job(self, aggregator: Any, oracle_account: OracleAccount, params: AggregatorSaveResultParams) -> TransactionSignature:
        return await self.program.provider.send(
            tx=(
                await self.save_result_txn(
                    aggregator,
                    oracle_account,
                    params
                )
            )
        )
    
    """
    RPC call for an oracle to save a result to an aggregator round.

    Args:
        aggregator (Any): Aggregator data
        oracle_account (OracleAccount)
        params (AggregatorSaveResultParams)

    Returns:
        TransactionSignature
    """
    async def save_result_txn(self, aggregator: Any, oracle_account: OracleAccount, params: AggregatorSaveResultParams):
        payer_keypair = Keypair.from_secret_key(self.program.provider.wallet.payer.secret_key)
        remaining_accounts: list[PublicKey] = []
        for i in range(aggregator.oracle_request_batch_size):
            remaining_accounts.append(aggregator.current_round.oracle_pubkeys_data[i])
        for oracle in params.oracles:
            remaining_accounts.push(oracle.token_account)
        queue_pubkey = aggregator.queue_pubkey
        queue_account = OracleQueueAccount(AccountParams(program=self.program, public_key=queue_pubkey))
        lease_account, lease_bump = LeaseAccount.from_seed(
            self.program,
            queue_account,
            self
        )
        escrow = get_associated_token_address(lease_account.public_key, params.token_mint)
        feed_permission_account, feed_permission_bump = PermissionAccount.from_seed(
            self.program,
            params.queue_authority,
            queue_account.public_key,
            self.public_key
        )
        oracle_permission_account, oracle_permission_bump = PermissionAccount.from_seed(
            self.program,
            params.queue_authority,
            queue_account.public_key,
            oracle_account.public_key
        )
        program_state_account, state_bump = ProgramStateAccount.from_seed(self.program)
        digest = await self.produce_job_hash(params.jobs).digest()
        history_buffer = aggregator.history_buffer
        if history_buffer == PublicKey('11111111111111111111111111111111'):
            history_buffer = self.public_key
        return self.program.transaction['aggregator_save_result'](
            {
                "oracle_idx": params.oracle_idx,
                "error": params.error,
                "value": SwitchboardDecimal.from_decimal(params.value).as_proper_sbd(self.program),
                "jobs_checksum": digest,
                "min_response": SwitchboardDecimal.from_decimal(params.min_response).as_proper_sbd(self.program),
                "max_response": SwitchboardDecimal.from_decimal(params.max_response).as_proper_sbd(self.program),
                "feed_permission_bump": feed_permission_bump,
                "oracle_permission_bump": oracle_permission_bump,
                "lease_bump": lease_bump,
                "state_bump": state_bump
            },
            ctx=anchorpy.Context(
                accounts={
                    "aggregator": self.public_key,
                    "oracle": oracle_account.public_key,
                    "oracle_authority": payer_keypair.public_key,
                    "oracle_queue": queue_account.public_key,
                    "feed_permission": feed_permission_account.public_key,
                    "oracle_permission": oracle_permission_account.public_key,
                    "lease": lease_account.public_key,
                    "escrow": escrow,
                    "token_program": TOKEN_PROGRAM_ID,
                    "program_state": program_state_account.public_key,
                    "history_buffer": history_buffer,
                    "mint": params.token_mint
                },
                remaining_accounts=[{"is_signer": False, "is_writable": True, "pubkey": pubkey} for pubkey in remaining_accounts]
            )
        )