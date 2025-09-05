# Agreement Registry Demo on SVM

## To Run

To install the prerequisites, follow [this instructions](https://solana.com/docs/intro/installation)

```shell
# To run the script (as tests)
anchor build
anchor test

# To verify the signature
solana verify-offchain-signature -k /path/to/solana/id.json '[{"key":"name","value":"Alice"},{"key":"age","value":"30"}]' <signature>
Signature is valid 
```
