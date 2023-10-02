# Token-2022 Standard Integration

We have introduced changes to incorporate the new Token-2022 standard for `spl-tokens` into our codebase. These changes bring additional functionalities, such as fee calculations, to ensure accurate payments for transactions involving tokens adhering to the Token-2022 standard.

## Key Enhancements

### 1. Fee Calculations

We have implemented a fee calculation mechanism to accurately calculate payments when dealing with tokens conforming to the Token-2022 standard. This ensures that users are charged the correct fees for their transactions involving these tokens.

### 2. Compatibility with Token-2022 Standard

Our codebase remains compatible with the regular token standard. The primary modification is the inclusion of the token mints as "remaining accounts." When using the program, ensure that the mint accounts are provided as remaining accounts.

### 3. Mint Accounts

The program expects the mint accounts to be passed as remaining accounts which are usually 2:

- **Base Mint Account**: This account represents the base token mint.
- **Quote Mint Account**: This account represents the quote token mint.

### 4. Token Program

To ensure compatibility with both the older token standard and the Token-2022 standard, it is essential to provide the correct `token_program` when interacting with our program. This ensures seamless execution of transactions involving different token standards.

## Note on Token Fees

For tokens adhering to the older standard, the token fee is set to 0 by default. This means that no token fee calculation occurs for transactions involving tokens conforming to this standard.

We added the token_fee calculation logic in the `token_utils` file, located at `programs/openbook-v2/src/token_utils.rs`. 

We believe these enhancements will provide a more robust and versatile experience for our users when working with different token standards.



### All tests so far have passed with these modifications


---

