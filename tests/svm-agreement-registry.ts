import * as anchor from "@coral-xyz/anchor";
import { BorshCoder, Idl, Program, web3 } from "@coral-xyz/anchor";
import { signBytes, createKeyPairFromBytes, generateKeyPair, getBase58Decoder } from "@solana/kit";
import { SvmAgreementRegistry } from "../target/types/svm_agreement_registry";
import { expect } from "chai"
import { ethers, TypedDataEncoder } from "ethers"

describe("svm-agreement-registry", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.svmAgreementRegistry as Program<SvmAgreementRegistry>;

  it("accepts Ed25519 signatures", async () => {
    const kvPairs: {key: string, value: string}[] = [
      { key: "name", value: "Alice"},
      { key: "age", value: "30"},
    ];

    // Format and sign the message

    const serializedKvPairs = serializeKeyValuePairs(kvPairs);

    // Construct the message preamble as per Solana CLI (https://github.com/anza-xyz/solana-sdk/blob/fed203a9e8b2a9d3b1a0c98c004e3d1ede569d0c/offchain-message/src/lib.rs#L184-L190)
    // TODO uncomment the missing fields once solana-cli has finally followed the spec (PR: https://github.com/anza-xyz/agave/issues/3340)
    const signingDomain = Buffer.from("\xffsolana offchain", "ascii");
    const headerVersion = Buffer.from([0]); // Version 0
    // const applicationDomain = Buffer.alloc(32); // Arbitrary 32-byte domain, can customize
    const messageFormat = Buffer.from([0]); // Message format 0 (printable chars)
    // const numSigners = Buffer.from([1]); // Single signer
    // const signerPubkey = provider.wallet.publicKey.toBuffer(); // 32-byte public key
    // const messageBytes = Buffer.from(message, "ascii");
    const messageLength = Buffer.alloc(2);
    messageLength.writeUInt16LE(serializedKvPairs.length); // Little-endian u16

    // Combine all parts into the formatted offchain message
    const offchainMessage = Buffer.concat([
      signingDomain,
      headerVersion,
      // applicationDomain,
      messageFormat,
      // numSigners,
      // signerPubkey,
      messageLength,
      serializedKvPairs,
    ]);

    // Sign it
    const keyPair = await createKeyPairFromBytes(provider.wallet.payer.secretKey);
    const signatureBytes = await signBytes(keyPair.privateKey, new Uint8Array(offchainMessage));
    const signature = Array.from(signatureBytes);

    // TODO test: uncomment to verify signature locally
    // const verified = await verifySignature(keyPair.publicKey, signedBytes, finalMessage);
    // console.log({ verified });

    const newDataEntryAccountKp = new web3.Keypair();
    const tx = (new web3.Transaction())
      // SvmAgreementRegistry will check if the previous instruction is a valid Ed25519 instruction, so we must call it
      // immediately before calling SvmAgreementRegistry.
      .add(
        anchor.web3.Ed25519Program.createInstructionWithPublicKey({
          // TODO Is seems to work but the error returned is weird:
          //  Error processing Instruction 0: custom program error: 0x2
          // publicKey: newDataEntryAccountKp.publicKey.toBytes(),
          // message: messageBytes,

          publicKey: provider.wallet.payer.publicKey.toBytes(),
          message: offchainMessage,

          signature: signatureBytes,
        })
      )
      .add(
        await program.methods
          .proposeAndSignAgreement(
            kvPairs,
            provider.wallet.publicKey,
            signature,
          )
          .accounts({
            dataEntry: newDataEntryAccountKp.publicKey,
            signer: provider.wallet.publicKey,
            sysvarIx: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .signers([newDataEntryAccountKp])
          .instruction()
      );

    // Send tx
    try {
      // Send and confirm the transaction
      const signature = await web3.sendAndConfirmTransaction(
        provider.connection, // Use provider's connection
        tx, // The transaction
        [
          provider.wallet.payer,
          newDataEntryAccountKp
        ], // Signers
        {
          commitment: "confirmed", // Wait for confirmation
        }
      );

      console.log(`Transaction successful with signature: ${signature}`);

      // Optionally, verify the transaction's effects
      const txDetails = await provider.connection.getTransaction(signature, {
        commitment: "confirmed",
      });
      expect(txDetails).to.not.be.null;
      // Add more assertions based on expected state changes
    } catch (error) {
      console.error("Transaction failed:", error);
      throw error; // Fail the test on error
    }
  });

  it("accepts secp256k1(EVM) signatures", async () => {
    const kvPairs: {key: string, value: string}[] = [
      { key: "name", value: "Alice"},
      { key: "age", value: "30"},
    ];

    const domain = {
      name: "CyberAgreementRegistry",
      version: "1",
      chainId: 1, // Ethereum mainnet
      verifyingContract: "0xa9E808B8eCBB60Bb19abF026B5b863215BC4c134"
    };

    const types = {
      SignatureData: [
        { name: "kvPairs", type: "KeyValuePair[]" },
      ],
      KeyValuePair: [
        { name: 'key', type: 'string' },
        { name: 'value', type: 'string' },
      ]
    };

    const value = {
      kvPairs,
    };

    const ethSigner = new ethers.Wallet(process.env.ETH_SIGNER_PRIVATE_KEY);

    const typedMessage = TypedDataEncoder.encode(domain, types, value);
    const typedMessageBytes: Uint8Array = ethers.getBytes(typedMessage);

    const fullSignature = await ethSigner.signTypedData(domain, types, value);
    const fullSignatureBytes = ethers.getBytes(fullSignature);
    const signatureBytes = fullSignatureBytes.slice(0, 64);
    const recoveryId = fullSignatureBytes[64] - 27;

    const newDataEntryAccountKp = new web3.Keypair();
    const tx = (new web3.Transaction())
      // SvmAgreementRegistry will check if the previous instruction is a valid secp256k1 instruction, so we must call it
      // immediately before calling SvmAgreementRegistry.
      .add(
        anchor.web3.Secp256k1Program.createInstructionWithEthAddress({
          message: typedMessageBytes,
          ethAddress: await ethSigner.getAddress(),
          signature: signatureBytes,
          recoveryId: recoveryId,
        })
      )
      .add(
        await program.methods
          .proposeAndSignAgreementEth(
            kvPairs,
            Array.from(ethers.getBytes(await ethSigner.getAddress())),
            Array.from(signatureBytes),
            Buffer.from(typedMessageBytes),
            recoveryId,
          )
          .accounts({
            dataEntry: newDataEntryAccountKp.publicKey,
            signer: provider.wallet.publicKey,
            sysvarIx: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .signers([newDataEntryAccountKp])
          .instruction()
      );

    // Send tx
    try {
      // Send and confirm the transaction
      const signature = await web3.sendAndConfirmTransaction(
        provider.connection, // Use provider's connection
        tx, // The transaction
        [
          provider.wallet.payer,
          newDataEntryAccountKp
        ], // Signers
        {
          commitment: "confirmed", // Wait for confirmation
        }
      );

      console.log(`Transaction successful with signature: ${signature}`);

      // Optionally, verify the transaction's effects
      const txDetails = await provider.connection.getTransaction(signature, {
        commitment: "confirmed",
      });
      expect(txDetails).to.not.be.null;
      // Add more assertions based on expected state changes
    } catch (error) {
      console.error("Transaction failed:", error);
      throw error; // Fail the test on error
    }
  });
});

function serializeKeyValuePairs(items: { key: string; value: string }[]): Uint8Array {
  const coder = new BorshCoder({
    "address": "11111111111111111111111111111111",
    "metadata": {
      "name": "placeholder",
      "version": "0.1.0",
      "spec": "0.1.0",
    },
    "instructions": [],
    "accounts": [],
    "types": [
      {
        "name": "KeyValuePairs",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "kvPairs",
              "type": {
                "vec": {
                  "defined": {
                    "name": "KeyValuePair"
                  }
                }
              }
            }
          ]
        }
      },
      {
        "name": "KeyValuePair",
        "type": {
          "kind": "struct",
          "fields": [
            {
              "name": "key",
              "type": "string"
            },
            {
              "name": "value",
              "type": "string"
            }
          ]
        }
      }
    ]
  });

  return coder.types.encode("KeyValuePairs", {
    kvPairs: items,
  });
}
