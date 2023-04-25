import * as anchor from "@project-serum/anchor";
import { AccountMeta, PublicKey, TokenAmount } from "@solana/web3.js";
import { OracleJob } from "@switchboard-xyz/common";
import {
  AggregatorAccount,
  BufferRelayerAccount,
  CrankAccount,
  CrankRow,
  JobAccount,
  LeaseAccount,
  OracleAccount,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  SwitchboardDecimal,
  SwitchboardPermissionValue,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import chalk from "chalk";
import { getIdlAddress, getProgramDataAddress } from "./anchor.js";
import { anchorBNtoDateTimeString } from "./date.js";
import { InvalidSwitchboardAccount } from "./errors.js";
import type { SwitchboardAccountType } from "./switchboard.js";

export const chalkString = (
  label: string,
  value: string | number | boolean | PublicKey | Big | anchor.BN,
  padding = 16
): string => {
  let valueString = "";
  if (typeof value === "string") {
    valueString = value;
  } else if (typeof value === "number") {
    valueString = value.toString();
  } else if (typeof value === "boolean") {
    valueString = value.toString();
  } else if (value instanceof PublicKey) {
    if (PublicKey.default.equals(value)) {
      valueString = "N/A";
    } else {
      valueString = value.toString();
    }
  } else if (value !== undefined) {
    valueString = value.toString();
  }
  return `${chalk.blue(label.padEnd(padding, " "))}${chalk.yellow(
    valueString
  )}`;
};

// JSON.stringify: Object => String
export const pubKeyConverter = (key: any, value: any): any => {
  if (value instanceof PublicKey || key.toLowerCase().endsWith("publickey")) {
    return value.toString() ?? "";
  }
  if (value instanceof Uint8Array) {
    return `[${value.toString()}]`;
  }
  if (value instanceof anchor.BN) {
    return value.toString();
  }
  if (value instanceof Big) {
    return value.toString();
  }
  if (value instanceof SwitchboardDecimal) {
    return new Big(value.mantissa.toString())
      .div(new Big(10).pow(value.scale))
      .toString();
  }
  return value;
};

export const tokenAmountString = (value: TokenAmount): string => {
  return `${value.uiAmountString ?? ""} (${value.amount})`;
};

/* eslint-disable no-control-regex */
export const buffer2string = (buf: Buffer | string | ArrayBuffer): string => {
  return Buffer.from(buf as any)
    .toString("utf8")
    .replace(/\u0000/g, ""); // removes padding from onchain fixed sized buffers
};

export const toPermissionString = (
  permission: SwitchboardPermissionValue
): string => {
  switch (permission) {
    case SwitchboardPermissionValue.PERMIT_ORACLE_HEARTBEAT:
      return "PERMIT_ORACLE_HEARTBEAT";
    case SwitchboardPermissionValue.PERMIT_ORACLE_QUEUE_USAGE:
      return "PERMIT_ORACLE_QUEUE_USAGE";
    case SwitchboardPermissionValue.PERMIT_VRF_REQUESTS:
      return "PERMIT_VRF_REQUESTS";
    default:
      return "NONE";
  }
};

export type VrfStatusString =
  | ""
  | "StatusNone"
  | "StatusRequesting"
  | "StatusVerifying"
  | "StatusVerified"
  | "StatusCallbackSuccess"
  | "StatusVerifyFailure";

export const toVrfStatusString = (
  status: Record<string, unknown>
): VrfStatusString => {
  try {
    if ("statusNone" in status) {
      return "StatusNone";
    }
    if ("statusRequesting" in status) {
      return "StatusRequesting";
    }
    if ("statusVerifying" in status) {
      return "StatusVerifying";
    }
    if ("statusVerified" in status) {
      return "StatusVerified";
    }
    if ("statusCallbackSuccess" in status) {
      return "StatusCallbackSuccess";
    }
    if ("statusVerifyFailure" in status) {
      return "StatusVerifyFailure";
    }
  } catch {}
  return "";
};

export async function prettyPrintProgramState(
  programState: ProgramStateAccount,
  accountData?: any,
  printIdlAddress = false,
  printDataAddress = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await programState.loadData());

  let outputString = "";
  outputString += chalk.underline(
    chalkString("## SbState", programState.publicKey, SPACING) + "\r\n"
  );
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString += chalkString("tokenMint", data.tokenMint, SPACING) + "\r\n";
  outputString += chalkString("tokenVault", data.tokenVault, SPACING) + "\r\n";
  outputString += chalkString("daoMint", data.daoMint, SPACING);

  if (printIdlAddress) {
    const idlAddress = await getIdlAddress(programState.program.programId);
    outputString += "\r\n" + chalkString("idlAddress", idlAddress, SPACING);
  }
  if (printDataAddress) {
    const dataAddress = getProgramDataAddress(programState.program.programId);
    outputString +=
      "\r\n" + chalkString("programDataAddress", dataAddress, SPACING);
  }

  return outputString;
}

export async function prettyPrintOracle(
  oracleAccount: OracleAccount,
  accountData?: any,
  printPermissions = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await oracleAccount.loadData());
  const oracleTokenAmount =
    await oracleAccount.program.provider.connection.getTokenAccountBalance(
      data.tokenAccount
    );

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Oracle", oracleAccount.publicKey, SPACING) + "\r\n"
  );
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("metadata", buffer2string(data.metadata as any), SPACING) +
    "\r\n";
  outputString +=
    chalkString(
      "balance",
      tokenAmountString(oracleTokenAmount.value),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString("oracleAuthority", data.oracleAuthority, SPACING) + "\r\n";
  outputString +=
    chalkString("tokenAccount", data.tokenAccount, SPACING) + "\r\n";
  outputString +=
    chalkString("queuePubkey", data.queuePubkey, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "lastHeartbeat",
      anchorBNtoDateTimeString(data.lastHeartbeat),
      SPACING
    ) + "\r\n";
  outputString += chalkString("numInUse", data.numInUse, SPACING) + "\r\n";
  outputString += chalkString(
    "metrics",
    JSON.stringify(data.metrics, undefined, 2),
    SPACING
  );

  if (printPermissions) {
    let permissionAccount: PermissionAccount;
    try {
      const queueAccount = new OracleQueueAccount({
        program: oracleAccount.program,
        publicKey: data.queuePubkey,
      });
      const queue = await queueAccount.loadData();
      [permissionAccount] = PermissionAccount.fromSeed(
        oracleAccount.program,
        queue.authority,
        queueAccount.publicKey,
        oracleAccount.publicKey
      );
      const permissionData = await permissionAccount.loadData();
      outputString +=
        "\r\n" +
        (await prettyPrintPermissions(permissionAccount, permissionData));
    } catch {
      outputString += `\r\nFailed to load permission account. Has it been created yet?`;
    }
  }

  return outputString;
}

export async function prettyPrintPermissions(
  permissionAccount: PermissionAccount,
  accountData?: any,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await permissionAccount.loadData());

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Permission", permissionAccount.publicKey, SPACING) + "\r\n"
  );
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString +=
    chalkString("permissions", toPermissionString(data.permissions), SPACING) +
    "\r\n";
  outputString += chalkString("granter", data.granter, SPACING) + "\r\n";
  outputString += chalkString("grantee", data.grantee, SPACING) + "\r\n";
  outputString += chalkString(
    "expiration",
    anchorBNtoDateTimeString(data.expiration),
    SPACING
  );
  return outputString;
}

export async function prettyPrintQueue(
  queueAccount: OracleQueueAccount,
  accountData?: any,
  printOracles = false,
  SPACING = 30
): Promise<string> {
  const data = accountData ?? (await queueAccount.loadData());

  const varianceToleranceMultiplier = SwitchboardDecimal.from(
    data.varianceToleranceMultiplier
  ).toBig();

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Queue", queueAccount.publicKey, SPACING) + "\r\n"
  );
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("metadata", buffer2string(data.metadata as any), SPACING) +
    "\r\n";
  outputString +=
    chalkString("oracleBuffer", data.dataBuffer, SPACING) + "\r\n";
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString +=
    chalkString("oracleTimeout", data.oracleTimeout, SPACING) + "\r\n";
  outputString += chalkString("reward", data.reward, SPACING) + "\r\n";
  outputString += chalkString("minStake", data.minStake, SPACING) + "\r\n";
  outputString +=
    chalkString("slashingEnabled", data.slashingEnabled, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "consecutiveFeedFailureLimit",
      data.consecutiveFeedFailureLimit.toString(),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "consecutiveOracleFailureLimit",
      data.consecutiveOracleFailureLimit.toString(),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "varianceToleranceMultiplier",
      varianceToleranceMultiplier,
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "feedProbationPeriod",
      data.feedProbationPeriod.toString(),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "unpermissionedFeedsEnabled",
      data.unpermissionedFeedsEnabled.toString(),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "unpermissionedVrfEnabled",
      data.unpermissionedVrfEnabled.toString(),
      SPACING
    ) + "\r\n";
  outputString += chalkString(
    "enableBufferRelayers",
    data.enableBufferRelayers?.toString() ?? "",
    SPACING
  );

  if (printOracles && data.queue) {
    outputString += chalk.underline(
      chalkString("\r\n## Oracles", " ".repeat(32), SPACING) + "\r\n"
    );
    outputString += (data.queue as PublicKey[])
      .filter((pubkey) => !PublicKey.default.equals(pubkey))
      .map((pubkey) => pubkey.toString())
      .join("\n");

    // (data.queue as PublicKey[]).forEach(
    //   (row, index) =>
    //     (outputString +=
    //       chalkString(`# ${index + 1},`, row.toString(), SPACING) + "\r\n")
    // );
  }

  return outputString;
}

export async function prettyPrintLease(
  leaseAccount: LeaseAccount,
  accountData?: any,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await leaseAccount.loadData());

  const escrowTokenAmount =
    await leaseAccount.program.provider.connection.getTokenAccountBalance(
      data.escrow
    );
  const balance = Number.parseInt(escrowTokenAmount.value.amount, 10);

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Lease", leaseAccount.publicKey, SPACING) + "\r\n"
  );
  outputString += chalkString("escrow", data.escrow, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "escrowBalance",
      tokenAmountString(escrowTokenAmount.value),
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString("withdrawAuthority", data.withdrawAuthority, SPACING) + "\r\n";
  outputString += chalkString("queue", data.queue, SPACING) + "\r\n";
  outputString += chalkString("aggregator", data.aggregator, SPACING) + "\r\n";
  outputString += chalkString("isActive", data.isActive, SPACING);

  return outputString;
}

export async function prettyPrintJob(
  jobAccount: JobAccount,
  accountData?: any,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await jobAccount.loadData());

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Job", jobAccount.publicKey, SPACING) + "\r\n"
  );
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("metadata", buffer2string(data.metadata as any), SPACING) +
    "\r\n";
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString += chalkString("expiration", data.expiration, SPACING) + "\r\n";
  outputString += chalkString(
    "tasks",
    JSON.stringify(OracleJob.decodeDelimited(data.data).tasks, undefined, 2),
    SPACING
  );

  return outputString;
}

// TODO: Add rest of fields
export async function prettyPrintAggregator(
  aggregatorAccount: AggregatorAccount,
  accountData?: any,
  printPermissions = false,
  printLease = false,
  printJobs = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await aggregatorAccount.loadData());

  const result = SwitchboardDecimal.from(data.latestConfirmedRound.result)
    .toBig()
    .toString();

  const resultTimestamp = anchorBNtoDateTimeString(
    data.latestConfirmedRound.roundOpenTimestamp ?? new anchor.BN(0)
  );

  const varianceThreshold = parseFloat(
    SwitchboardDecimal.from(data.varianceThreshold).toBig().toString()
  ).toFixed(2);

  let outputString = "";
  outputString += chalk.underline(
    chalkString(
      "## Aggregator",
      aggregatorAccount.publicKey ?? PublicKey.default,
      SPACING
    ) + "\r\n"
  );

  outputString +=
    chalkString(
      "latestResult",
      `${result} (${resultTimestamp ?? ""})`,
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("metadata", buffer2string(data.metadata as any), SPACING) +
    "\r\n";
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString +=
    chalkString("queuePubkey", data.queuePubkey, SPACING) + "\r\n";
  outputString +=
    chalkString("crankPubkey", data.crankPubkey, SPACING) + "\r\n";
  outputString +=
    chalkString("historyBufferPublicKey", data.historyBuffer, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "authorWallet",
      data.authorWallet ?? PublicKey.default,
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString("minUpdateDelaySeconds", data.minUpdateDelaySeconds, SPACING) +
    "\r\n";
  outputString +=
    chalkString("jobPubkeysSize", data.jobPubkeysSize, SPACING) + "\r\n";
  outputString +=
    chalkString("minJobResults", data.minJobResults, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "oracleRequestBatchSize",
      data.oracleRequestBatchSize,
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString("minOracleResults", data.minOracleResults, SPACING) + "\r\n";
  outputString +=
    chalkString("varianceThreshold", `${varianceThreshold} %`, SPACING) +
    "\r\n";
  outputString +=
    chalkString("forceReportPeriod", data.forceReportPeriod, SPACING) + "\r\n";
  outputString += chalkString("isLocked", data.isLocked, SPACING);

  if (printPermissions) {
    let permissionAccount: PermissionAccount;
    try {
      const queueAccount = new OracleQueueAccount({
        program: aggregatorAccount.program,
        publicKey: data.queuePubkey,
      });
      const queue = await queueAccount.loadData();
      [permissionAccount] = PermissionAccount.fromSeed(
        aggregatorAccount.program,
        queue.authority,
        queueAccount.publicKey,
        aggregatorAccount.publicKey ?? PublicKey.default
      );
      const permissionData = await permissionAccount.loadData();
      outputString +=
        "\r\n" +
        (await prettyPrintPermissions(permissionAccount, permissionData));
    } catch {
      outputString += `\r\nFailed to load permission account. Has it been created yet?`;
    }
  }

  if (printLease) {
    let leaseAccount: LeaseAccount;
    try {
      const queueAccount = new OracleQueueAccount({
        program: aggregatorAccount.program,
        publicKey: data.queuePubkey,
      });
      const { authority } = await queueAccount.loadData();
      [leaseAccount] = LeaseAccount.fromSeed(
        aggregatorAccount.program,
        queueAccount,
        aggregatorAccount
      );
      const leaseData = await leaseAccount.loadData();
      outputString +=
        "\r\n" + (await prettyPrintLease(leaseAccount, leaseData));
    } catch {
      outputString += `\r\nFailed to load lease account. Has it been created yet?`;
    }
  }

  if (printJobs) {
    const jobKeys: PublicKey[] = (data.jobPubkeysData as PublicKey[]).filter(
      (pubkey) => !PublicKey.default.equals(pubkey)
    );
    for await (const jobKey of jobKeys) {
      const jobAccount = new JobAccount({
        program: aggregatorAccount.program,
        publicKey: jobKey,
      });
      outputString += "\r\n" + (await prettyPrintJob(jobAccount));
    }
  }

  return outputString;
}

export async function prettyPrintVrf(
  vrfAccount: VrfAccount,
  accountData?: any,
  printPermissions = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await vrfAccount.loadData());
  const escrowTokenAmount =
    await vrfAccount.program.provider.connection.getTokenAccountBalance(
      data.escrow
    );

  let outputString = "";
  outputString += chalk.underline(
    chalkString("## VRF", vrfAccount.publicKey, SPACING) + "\r\n"
  );
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString +=
    chalkString("oracleQueue", data.oracleQueue, SPACING) + "\r\n";
  outputString += chalkString("escrow", data.escrow, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "escrowBalance",
      tokenAmountString(escrowTokenAmount.value),
      SPACING
    ) + "\r\n";

  outputString += chalkString("batchSize", data.batchSize, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "callback",
      JSON.stringify(
        {
          ...data.callback,
          accounts: data.callback.accounts.filter(
            (a: AccountMeta) => !a.pubkey.equals(PublicKey.default)
          ),
          ixData: `[${data.callback.ixData
            .slice(0, data.callback.ixDataLen)
            .map((n) => n.toString())
            .join(",")}]`,
        },
        undefined,
        2
      ),
      SPACING
    ) + "\r\n";
  outputString += chalkString("counter", data.counter, SPACING) + "\r\n";
  outputString +=
    chalkString("status", toVrfStatusString(data.status), SPACING) + "\r\n";
  outputString += chalkString(
    "latestResult",
    JSON.stringify(
      {
        status: toVrfStatusString(data.builders[0]?.status) ?? "",
        verified: data.builders[0]?.verified ?? "",
        txRemaining: data.builders[0]?.txRemaining ?? "",
        producer: data.builders[0]?.producer.toString() ?? "",
        reprProof: data.builders[0].reprProof
          ? `[${data.builders[0].reprProof.map((value) => value.toString())}]`
          : "",
        reprProofHex: data.builders[0].reprProof
          ? Buffer.from(data.builders[0].reprProof).toString("hex")
          : "",
        currentRound: {
          result: data.currentRound.result
            ? `[${data.currentRound.result.map((value) => value.toString())}]`
            : "",
          alpha: data.currentRound.alpha
            ? `[${data.currentRound.alpha.map((value) => value.toString())}]`
            : "",
          alphaHex: Buffer.from(data.currentRound.alpha).toString("hex"),
          requestSlot: data.currentRound?.requestSlot?.toString() ?? "",
          requestTimestamp: anchorBNtoDateTimeString(
            data.currentRound.requestTimestamp
          ),
          numVerified: data.currentRound.numVerified.toString(),
        },
      },
      undefined,
      2
    ),
    SPACING
  );

  if (printPermissions) {
    let permissionAccount: PermissionAccount;
    try {
      const queueAccount = new OracleQueueAccount({
        program: vrfAccount.program,
        publicKey: data.oracleQueue,
      });
      const queue = await queueAccount.loadData();
      [permissionAccount] = PermissionAccount.fromSeed(
        vrfAccount.program,
        queue.authority,
        queueAccount.publicKey,
        vrfAccount.publicKey
      );
      const permissionData = await permissionAccount.loadData();
      outputString +=
        "\r\n" +
        (await prettyPrintPermissions(permissionAccount, permissionData));
    } catch {
      outputString += `\r\nFailed to load permission account. Has it been created yet?`;
    }
  }

  return outputString;
}

export async function prettyPrintCrank(
  crankAccount: CrankAccount,
  accountData?: any,
  printRows = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await crankAccount.loadData());

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## Crank", crankAccount.publicKey, SPACING) + "\r\n"
  );
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("metadata", buffer2string(data.metadata as any), SPACING) +
    "\r\n";
  outputString +=
    chalkString("queuePubkey", data.queuePubkey, SPACING) + "\r\n";
  outputString += chalkString("dataBuffer", data.dataBuffer, SPACING) + "\r\n";
  outputString +=
    chalkString(
      "Size",
      `${(data.pqData as CrankRow[]).length
        .toString()
        .padStart(4)} / ${data.maxRows.toString().padEnd(4)}`,
      SPACING
    ) + "\r\n";

  if (printRows) {
    outputString += chalk.underline(
      chalkString("## Crank Buffer", data.dataBuffer, SPACING) + "\r\n"
    );
    const rowStrings = data.pqData.map((row) => {
      return `${anchorBNtoDateTimeString(row.nextTimestamp as anchor.BN).padEnd(
        16
      )} - ${(row.pubkey as PublicKey).toString()}`;
    });
    outputString = outputString.concat(...rowStrings.join("\n"));

    // const feedNames: string[] = [];
    // for await (const row of data.pqData) {
    //   const agg = new AggregatorAccount({
    //     program: crankAccount.program,
    //     publicKey: row.pubkey,
    //   });
    //   const aggData = await agg.loadData();
    //   const aggName = buffer2string(aggData.name as any);
    //   feedNames.push(`${(row.pubkey as PublicKey).toString()} # ${aggName}`);
    // }

    // outputString = outputString.concat("\n", ...feedNames.join("\n"));
  }
  return outputString;
}

export async function prettyPrintBufferRelayer(
  bufferRelayerAccount: BufferRelayerAccount,
  accountData?: any,
  printJob = false,
  SPACING = 24
): Promise<string> {
  const data = accountData ?? (await bufferRelayerAccount.loadData());

  let outputString = "";

  outputString += chalk.underline(
    chalkString("## BufferRelayer", bufferRelayerAccount.publicKey, SPACING) +
      "\r\n"
  );
  outputString +=
    chalkString("name", buffer2string(data.name as any), SPACING) + "\r\n";
  outputString +=
    chalkString("queuePubkey", data.queuePubkey, SPACING) + "\r\n";
  outputString += chalkString("escrow", data.escrow, SPACING) + "\r\n";
  outputString += chalkString("authority", data.authority, SPACING) + "\r\n";
  outputString += chalkString("jobPubkey", data.jobPubkey, SPACING) + "\r\n";
  outputString +=
    chalkString("minUpdateDelaySeconds", data.minUpdateDelaySeconds, SPACING) +
    "\r\n";

  const result = data.result as number[];
  outputString +=
    chalkString(
      "result",
      `[${result.map((r) => r.toString()).join(",")}]`,
      SPACING
    ) + "\r\n";
  outputString +=
    chalkString(
      "currentRound",
      JSON.stringify(data.currentRound, pubKeyConverter, 2),
      SPACING
    ) + "\r\n";

  if (printJob) {
    const jobAccount = new JobAccount({
      program: bufferRelayerAccount.program,
      publicKey: data.jobPubkey,
    });
    outputString += "\r\n" + (await prettyPrintJob(jobAccount));
  }

  return outputString;
}

export async function prettyPrintSwitchboardAccount(
  program: anchor.Program,
  publicKey: PublicKey,
  accountType: SwitchboardAccountType
): Promise<string> {
  switch (accountType) {
    case JobAccount.accountName: {
      const job = new JobAccount({ program, publicKey });
      return prettyPrintJob(job);
    }
    case AggregatorAccount.accountName: {
      const aggregator = new AggregatorAccount({ program, publicKey });
      return prettyPrintAggregator(aggregator, undefined);
    }
    case OracleAccount.accountName: {
      const oracle = new OracleAccount({ program, publicKey });
      return prettyPrintOracle(oracle, undefined);
    }
    case PermissionAccount.accountName: {
      const permission = new PermissionAccount({ program, publicKey });
      return prettyPrintPermissions(permission, undefined);
    }
    case LeaseAccount.accountName: {
      const lease = new LeaseAccount({ program, publicKey });
      return prettyPrintLease(lease, undefined);
    }
    case OracleQueueAccount.accountName: {
      const queue = new OracleQueueAccount({ program, publicKey });
      return prettyPrintQueue(queue, undefined);
    }
    case CrankAccount.accountName: {
      const crank = new CrankAccount({ program, publicKey });
      return prettyPrintCrank(crank, undefined);
    }
    case "SbState":
    case ProgramStateAccount.accountName: {
      const [programState] = ProgramStateAccount.fromSeed(program);
      return prettyPrintProgramState(programState);
    }
    case VrfAccount.accountName: {
      const vrfAccount = new VrfAccount({ program, publicKey });
      return prettyPrintVrf(vrfAccount, undefined);
    }
    case BufferRelayerAccount.accountName: {
      const bufferRelayerAccount = new BufferRelayerAccount({
        program,
        publicKey,
      });
      return prettyPrintBufferRelayer(bufferRelayerAccount, undefined);
    }
    case "BUFFERxx": {
      return `Found buffer account but dont know which one`;
    }
  }
  throw new InvalidSwitchboardAccount();
}
