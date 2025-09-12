use anchor_lang::solana_program::sysvar;
use anyhow::Result;
use anchor_client::{
    Client, Cluster,
    solana_sdk::signature::{read_keypair_file, Keypair, Signer},
};
use solana_commitment_config::CommitmentConfig;
use solana_ed25519_program::new_ed25519_instruction_with_signature;
use svm_agreement_registry::utils::ed25519::{KeyValuePair, format_message};

#[test]
fn test_accepts_ed25519_signatures() -> Result<()> {
    let anchor_wallet = std::env::var("ANCHOR_WALLET")?;
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
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

    let offchain_message = format_message(&kv_pairs)?;

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
                    kv_pairs,
                    signer: svm_agreement_registry_program.payer(),
                    signature: signature.into(),
                })
                .instructions()?
                .remove(0)
        )
        .signer(&data_entry)
        .send()?;

    println!("Your transaction signature {}", tx);
    Ok(())
}

// #[tokio::test]
// async fn test_accepts_secp256k1_signatures() {
//     let (mut context, payer, data_entry_pubkey) = setup().await;
//
//     let kv_pairs = vec![
//         KeyValuePair {
//             key: "name".to_string(),
//             value: "Alice".to_string(),
//         },
//         KeyValuePair {
//             key: "age".to_string(),
//             value: "30".to_string(),
//         },
//     ];
//
//     // For secp256k1, we'll use a simple message hash since we can't use ethers in Rust tests
//     let message = b"test message";
//     let eth_address = [1u8; 20]; // Mock Ethereum address
//     let signature = [2u8; 64]; // Mock signature
//     let recovery_id = 0;
//
//     // Create secp256k1 instruction
//     let secp256k1_instruction = Instruction {
//         program_id: SECP256K1_ID,
//         accounts: vec![],
//         data: [
//             signature.to_vec(),
//             vec![recovery_id],
//             vec![0u8], // message_data_offset
//             vec![0u8], // message_data_size
//             (message.len() as u16).to_le_bytes().to_vec(),
//             message.to_vec(),
//             eth_address.to_vec(),
//         ]
//             .concat(),
//     };
//
//     // Create program instruction
//     let program_instruction = Instruction {
//         program_id: svm_agreement_registry::ID,
//         accounts: svm_agreement_registry::accounts::ProposeAndSignAgreementEth {
//             data_entry: data_entry_pubkey,
//             signer: payer.pubkey(),
//             sysvar_ix: sysvar::instructions::ID,
//             system_program: system_program::ID,
//         }
//             .to_account_metas(Some(true)),
//         data: ProposeAndSignAgreementEth {
//             kv_pairs,
//             eth_address: eth_address.to_vec(),
//             signature: signature.to_vec(),
//             recovery_id,
//         }
//             .data(),
//     };
//
//     // Create and send transaction
//     let transaction = Transaction::new_signed_with_payer(
//         &[secp256k1_instruction, program_instruction],
//         Some(&payer.pubkey()),
//         &[&payer],
//         context.last_blockhash,
//     );
//
//     let result = context
//         .banks_client
//         .process_transaction(transaction)
//         .await;
//
//     assert!(result.is_ok(), "Transaction failed: {:?}", result.err());
// }
