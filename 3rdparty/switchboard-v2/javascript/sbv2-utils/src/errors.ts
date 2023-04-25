export class NoPayerKeypairProvided extends Error {
  constructor(message = "no payer keypair provided") {
    super(message);
    Object.setPrototypeOf(this, NoPayerKeypairProvided.prototype);
  }
}

export class InvalidSwitchboardAccount extends Error {
  constructor(message = "failed to match account type by discriminator") {
    super(message);
    Object.setPrototypeOf(this, InvalidSwitchboardAccount.prototype);
  }
}
