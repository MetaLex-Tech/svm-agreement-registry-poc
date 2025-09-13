use std::{
    env,
    rc::Rc,
};
use alloy::{
    signers::{SignerSync, local::PrivateKeySigner},
    primitives::keccak256,
};
use anchor_lang::solana_program::sysvar;
use anyhow::Result;
use anchor_client::{Client, Cluster, solana_sdk::signature::{read_keypair_file, Keypair, Signer}, Program};
use solana_commitment_config::CommitmentConfig;
use solana_ed25519_program::new_ed25519_instruction_with_signature;
use solana_secp256k1_program::new_secp256k1_instruction_with_signature;
use svm_agreement_registry::{
    DataEntry,
    utils::ed25519::{KeyValuePair, format_message as format_ed25519_message},
    utils::secp256k1::{format_message as format_secp256k1_message},
};

fn setup() -> Result<(Program<Rc<Keypair>>, Rc<Keypair>, Keypair, Vec<KeyValuePair>)> {
    let anchor_wallet = std::env::var("ANCHOR_WALLET")?;
    let payer = Rc::new(read_keypair_file(&anchor_wallet).unwrap());
    let client = Client::new_with_options(Cluster::Localnet, payer.clone(), CommitmentConfig::confirmed());
    let svm_agreement_registry_program = client.program(svm_agreement_registry::ID)?;

    let data_entry = Keypair::new();

    let kv_pairs = vec![
        KeyValuePair {
            key: "name".to_string(),
            value: "Alice".to_string(),
        },
        KeyValuePair {
            key: "age".to_string(),
            value: "30".to_string(),
        },
    ];

    Ok((svm_agreement_registry_program, payer, data_entry, kv_pairs))
}

#[test]
fn test_accepts_ed25519_signatures() -> Result<()> {
    let (svm_agreement_registry_program, payer, data_entry, kv_pairs) = setup()?;

    let offchain_message = format_ed25519_message(&kv_pairs)?;

    let signature = payer.sign_message(&offchain_message);

    let tx = svm_agreement_registry_program
        .request()
        .instruction(
            new_ed25519_instruction_with_signature(
                &offchain_message,
                signature.as_array(),
                &payer.pubkey().to_bytes(),
            ),
        )
        .instruction(
            svm_agreement_registry_program
                .request()
                .accounts(svm_agreement_registry::accounts::StoreData {
                    data_entry: data_entry.pubkey(),
                    signer: svm_agreement_registry_program.payer(),
                    system_program: solana_system_interface::program::ID,
                    sysvar_ix: sysvar::instructions::ID,
                })
                .args(svm_agreement_registry::instruction::ProposeAndSignAgreement {
                    kv_pairs: kv_pairs.clone(),
                    signer: svm_agreement_registry_program.payer(),
                    signature: signature.into(),
                })
                .instructions()?
                .remove(0)
        )
        .signer(&data_entry)
        .send()?;

    println!("Transaction signature: {}", tx);

    let data_entry_account: DataEntry = svm_agreement_registry_program.account(data_entry.pubkey())?;
    assert_eq!(data_entry_account.kv_pairs, kv_pairs);
    assert_eq!(data_entry_account.signer, payer.pubkey());

    Ok(())
}

#[test]
fn test_accepts_secp256k1_signatures() -> Result<()> {
    let (svm_agreement_registry_program, payer, data_entry, kv_pairs) = setup()?;

    let offchain_message = format_secp256k1_message(&kv_pairs)?;

    let eth_signer = env::var("ETH_SIGNER_PRIVATE_KEY")?.parse::<PrivateKeySigner>()?;
    let eth_address = eth_signer.address();
    let full_signature = eth_signer.sign_hash_sync(&keccak256(&offchain_message))?;
    let signature = full_signature.as_erc2098();
    let recovery_id = full_signature.recid().to_byte();

    let tx = svm_agreement_registry_program
        .request()
        .instruction(
            new_secp256k1_instruction_with_signature(
                &offchain_message,
                &signature,
                recovery_id,
                &eth_address.into_array(),
            ),
        )
        .instruction(
            svm_agreement_registry_program
                .request()
                .accounts(svm_agreement_registry::accounts::StoreData {
                    data_entry: data_entry.pubkey(),
                    signer: svm_agreement_registry_program.payer(),
                    system_program: solana_system_interface::program::ID,
                    sysvar_ix: sysvar::instructions::ID,
                })
                .args(svm_agreement_registry::instruction::ProposeAndSignAgreementEth {
                    kv_pairs: kv_pairs.clone(),
                    signer: eth_address.into_array(),
                    signature,
                    recovery_id,
                })
                .instructions()?
                .remove(0)
        )
        .signer(&data_entry)
        .send()?;

    println!("Transaction signature: {}", tx);

    let data_entry_account: DataEntry = svm_agreement_registry_program.account(data_entry.pubkey())?;
    assert_eq!(data_entry_account.kv_pairs, kv_pairs);
    assert_eq!(data_entry_account.signer.to_bytes()[12..32], eth_address.into_array());

    Ok(())
}
