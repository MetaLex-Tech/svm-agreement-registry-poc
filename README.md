# Agreement Registry Demo on SVM

The `svm_agreement_registry` is a Solana program implementing a secure key-value store protected by 
cryptographic signatures. Only data signed by the author is accepted. It supports 
both SVM's Ed25519 and EVM's secp256k1 signatures, facilitated through two entrypoints: 
`propose_and_sign_agreement` and `propose_and_sign_agreement_eth`, respectively.

## To Run

To install the prerequisites, follow [this instructions](https://solana.com/docs/intro/installation)

### Environment Variables
```shell
# Required
ETH_SIGNER_PRIVATE_KEY=0x...
```

```shell
# To run the tests
yarn
anchor test

# To verify the signature off-chain

# ed25519 signatures
solana verify-offchain-signature -k /path/to/solana/id.json '[{"key":"name","value":"Alice"},{"key":"age","value":"30"}]' <signature>
Signature is valid

# secp256k1 (ETH) signatures
# Unfortunately `cast wallet verify` does not verify EIP-712 signatures, so all we can to is sign again here and compare two output signatures
cast wallet sign --private-key <your-wallet-pk> --data '{"domain":{"name":"CyberAgreementRegistry","version":"1","chainId":1,"verifyingContract":"0xa9E808B8eCBB60Bb19abF026B5b863215BC4c134"},"message":{"kvPairs":[{"key":"name","value":"Alice"},{"key":"age","value":"30"}]},"primaryType":"SignatureData","types":{"SignatureData":[{"name":"kvPairs","type":"KeyValuePair[]"}],"KeyValuePair":[{"name":"key","type":"string"},{"name":"value","type":"string"}]}}'
0x123...
# Compare against `anchor test` outputs
```
