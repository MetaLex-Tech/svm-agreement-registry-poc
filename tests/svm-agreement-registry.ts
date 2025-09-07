import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { signBytes, createKeyPairFromBytes, generateKeyPair, getBase58Decoder } from "@solana/kit";
import { SvmAgreementRegistry } from "../target/types/svm_agreement_registry";
import { expect } from "chai"

describe("svm-agreement-registry", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.svmAgreementRegistry as Program<SvmAgreementRegistry>;

  it("stores data", async () => {
    const kvPairs: {key: string, value: string}[] = [
      { key: "name", value: "Alice"},
      { key: "age", value: "30"},
    ];

    // Generate keypair for the new account
    const newAccountKp = new web3.Keypair();

    // Sign the message
    const message = JSON.stringify(kvPairs);

    // Construct the message preamble as per Solana CLI (https://github.com/anza-xyz/solana-sdk/blob/fed203a9e8b2a9d3b1a0c98c004e3d1ede569d0c/offchain-message/src/lib.rs#L184-L190)
    // TODO uncomment the missing fields once they've finally followed the spec (PR: https://github.com/anza-xyz/agave/issues/3340)
    const signingDomain = Buffer.from("\xffsolana offchain", "ascii");
    const headerVersion = Buffer.from([0]); // Version 0
    // const applicationDomain = Buffer.alloc(32); // Arbitrary 32-byte domain, can customize
    const messageFormat = Buffer.from([0]); // Message format 0 (printable chars)
    // const numSigners = Buffer.from([1]); // Single signer
    // const signerPubkey = provider.wallet.publicKey.toBuffer(); // 32-byte public key
    const messageBytes = Buffer.from(message, "ascii");
    const messageLength = Buffer.alloc(2);
    messageLength.writeUInt16LE(messageBytes.length); // Little-endian u16

    // Combine all parts into the final message
    const finalMessage = Buffer.concat([
      signingDomain,
      headerVersion,
      // applicationDomain,
      messageFormat,
      // numSigners,
      // signerPubkey,
      messageLength,
      messageBytes,
    ]);

    // const messageBytes = new TextEncoder().encode(message);
    const keyPair = await createKeyPairFromBytes(provider.wallet.payer.secretKey);
    const signedBytes = await signBytes(keyPair.privateKey, new Uint8Array(finalMessage));
    const signature = Array.from(signedBytes);

    // TODO test
    // const publicKeyBytes = await crypto.subtle.exportKey('raw', keyPair.publicKey);
    // console.log("publicKey:", getBase58Decoder().decode(new Uint8Array(publicKeyBytes)));
    // console.log("publicKey:", Array.from(provider.wallet.publicKey.toBuffer()));

    console.log("message:", message);
    // console.log("finalMessage:", finalMessage);
    console.log("signature:", getBase58Decoder().decode(signedBytes));
    
    // TODO test: verify signature
    // const verified = await verifySignature(keyPair.publicKey, signedBytes, finalMessage);
    // console.log({ verified });

    const tx = (new web3.Transaction())
      // SvmAgreementRegistry will check if the previous instruction is a valid Ed25519 instruction, so we must call it
      // immediately before calling SvmAgreementRegistry.
      .add(
        anchor.web3.Ed25519Program.createInstructionWithPublicKey({
          // TODO Is seems to work but the error returned is weird:
          //  Error processing Instruction 0: custom program error: 0x2
          // publicKey: newAccountKp.publicKey.toBytes(),
          // message: messageBytes,
          publicKey: provider.wallet.payer.publicKey.toBytes(),
          message: finalMessage,

          signature: signedBytes,
        })
      )
      .add(
        await program.methods
          .storeData(
            kvPairs,
            provider.wallet.publicKey,
            signature,
            finalMessage,
          )
          .accounts({
            dataEntry: newAccountKp.publicKey,
            signer: provider.wallet.publicKey,
            sysvarIx: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .signers([newAccountKp])
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
          newAccountKp
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
