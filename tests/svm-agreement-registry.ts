import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { signBytes, createKeyPairFromBytes, verifySignature, getBase58Decoder } from "@solana/kit";
import { SvmAgreementRegistry } from "../target/types/svm_agreement_registry";

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
    const finalMessage = new Uint8Array(Buffer.concat([
      signingDomain,
      headerVersion,
      // applicationDomain,
      messageFormat,
      // numSigners,
      // signerPubkey,
      messageLength,
      messageBytes,
    ]));

    // const messageBytes = new TextEncoder().encode(message);
    const keyPair = await createKeyPairFromBytes(provider.wallet.payer.secretKey);
    const signedBytes = await signBytes(keyPair.privateKey, finalMessage);
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

    const tx = await program.methods
      .storeData(
        kvPairs,
        signature,
      )
      .accounts({
        dataEntry: newAccountKp.publicKey,
        signer: provider.wallet.publicKey,
      })
      .signers([newAccountKp])
      .rpc();

    // console.log("Your transaction signature", tx);
  });
});
