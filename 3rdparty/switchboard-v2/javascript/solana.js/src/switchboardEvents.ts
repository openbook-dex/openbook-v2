import * as anchor from '@project-serum/anchor';
import * as types from './generated';

export type SwitchboardEvents = {
  AggregatorInitEvent: {
    feedPubkey: anchor.web3.PublicKey;
  };
  VrfRequestRandomnessEvent: {
    vrfPubkey: anchor.web3.PublicKey;
    oraclePubkeys: anchor.web3.PublicKey[];
    loadAmount: anchor.BN;
    existingAmount: anchor.BN;
  };
  VrfRequestEvent: {
    vrfPubkey: anchor.web3.PublicKey;
    oraclePubkeys: anchor.web3.PublicKey[];
  };
  VrfProveEvent: {
    vrfPubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
    authorityPubkey: anchor.web3.PublicKey;
  };
  VrfVerifyEvent: {
    vrfPubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
    authorityPubkey: anchor.web3.PublicKey;
    amount: anchor.BN;
  };
  VrfCallbackPerformedEvent: {
    vrfPubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
    amount: anchor.BN;
  };
  AggregatorOpenRoundEvent: {
    feedPubkey: anchor.web3.PublicKey;
    oraclePubkeys: anchor.web3.PublicKey[];
    jobPubkeys: anchor.web3.PublicKey[];
    remainingFunds: anchor.BN;
    queueAuthority: anchor.web3.PublicKey;
  };
  AggregatorValueUpdateEvent: {
    feedPubkey: anchor.web3.PublicKey;
    value: types.SwitchboardDecimalFields;
    slot: anchor.BN;
    timestamp: anchor.BN;
    oraclePubkeys: anchor.web3.PublicKey[];
    oracleValues: types.SwitchboardDecimalFields[];
  };
  OracleRewardEvent: {
    feedPubkey: anchor.web3.PublicKey;
    leasePubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
    walletPubkey: anchor.web3.PublicKey;
    amount: anchor.BN;
    roundSlot: anchor.BN;
    timestamp: anchor.BN;
  };
  OracleWithdrawEvent: {
    oraclePubkey: anchor.web3.PublicKey;
    walletPubkey: anchor.web3.PublicKey;
    destinationWallet: anchor.web3.PublicKey;
    previousAmount: anchor.BN;
    newAmount: anchor.BN;
    timestamp: anchor.BN;
  };
  LeaseWithdrawEvent: {
    leasePubkey: anchor.web3.PublicKey;
    walletPubkey: anchor.web3.PublicKey;
    previousAmount: anchor.BN;
    newAmount: anchor.BN;
    timestamp: anchor.BN;
  };
  OracleSlashEvent: {
    feedPubkey: anchor.web3.PublicKey;
    leasePubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
    walletPubkey: anchor.web3.PublicKey;
    amount: anchor.BN;
    roundSlot: anchor.BN;
    timestamp: anchor.BN;
  };
  LeaseFundEvent: {
    leasePubkey: anchor.web3.PublicKey;
    funder: anchor.web3.PublicKey;
    amount: anchor.BN;
    timestamp: anchor.BN;
  };
  ProbationBrokenEvent: {
    feedPubkey: anchor.web3.PublicKey;
    queuePubkey: anchor.web3.PublicKey;
    timestamp: anchor.BN;
  };
  FeedPermissionRevokedEvent: {
    feedPubkey: anchor.web3.PublicKey;
    timestamp: anchor.BN;
  };
  GarbageCollectFailureEvent: {
    queuePubkey: anchor.web3.PublicKey;
  };
  OracleBootedEvent: {
    queuePubkey: anchor.web3.PublicKey;
    oraclePubkey: anchor.web3.PublicKey;
  };
  AggregatorCrankEvictionEvent: {};
  CrankLeaseInsufficientFundsEvent: {
    feedPubkey: anchor.web3.PublicKey;
    leasePubkey: anchor.web3.PublicKey;
  };
  CrankPopExpectedFailureEvent: {
    feedPubkey: anchor.web3.PublicKey;
    leasePubkey: anchor.web3.PublicKey;
  };
  BufferRelayerOpenRoundEvent: {
    relayerPubkey: anchor.web3.PublicKey;
    jobPubkey: anchor.web3.PublicKey;
    oraclePubkeys: anchor.web3.PublicKey[];
    remainingFunds: anchor.BN;
    queue: anchor.web3.PublicKey;
  };
};
