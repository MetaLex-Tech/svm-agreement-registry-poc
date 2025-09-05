import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { signBytes, createKeyPairFromBytes, verifySignature } from "@solana/kit";
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

    console.log("provider.wallet.publicKey:", provider.wallet.publicKey);

    // Generate keypair for the new account
    const newAccountKp = new web3.Keypair();
    console.log("newAccountKp.publicKey:", newAccountKp.publicKey);

    // Sign the message
    const message = JSON.stringify(kvPairs);
    const messageBytes = new TextEncoder().encode(message);
    const keyPair = await createKeyPairFromBytes(provider.wallet.payer.secretKey);
    const signedBytes = await signBytes(keyPair.privateKey, messageBytes);
    const signature = Array.from(signedBytes);

    // TODO test: verify signature
    // const verified = await verifySignature(keyPair.publicKey, signedBytes, messageBytes);
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

    console.log("Your transaction signature", tx);
  });
});
