use anchor_lang::prelude::*;
use solana_program::sysvar::instructions::{
    ID as SYSVAR_IX_ID,
    load_current_index_checked,
    load_instruction_at_checked,
};
use solana_program::{keccak, secp256k1_program};
use hex_literal::hex;
use crate::utils::ed25519::KeyValuePair;

pub fn verify_signature(
    ix_sysvar_account_info: &AccountInfo,
    signer: [u8; 20],
    signature: [u8; 64],
    recovery_id: u8,
    message: Vec<u8>,
) -> Result<()> {
    // Verify and extract the prior instruction (presumably an Secp256k1Program instruction for signature verification)
    let current_index = load_current_index_checked(&ix_sysvar_account_info)?;
    if current_index == 0 {
        return Err(error!(ErrorCode::MissingSecp256k1Instruction));
    }
    let secp256k1_instruction = load_instruction_at_checked((current_index - 1) as usize, &ix_sysvar_account_info)?;

    // Verify it is a valid Secp256k1Program instruction
    if secp256k1_instruction.program_id != secp256k1_program::id() {
        return Err(error!(ErrorCode::InvalidSecp256k1Program));
    }

    let secp256k1_instruction_layout = Secp256k1InstructionLayout::try_from_slice(&secp256k1_instruction.data[0..97])?;

    // Verify number of signatures
    if secp256k1_instruction_layout.num_signatures != 1 {
        return Err(error!(ErrorCode::InvalidSecp256k1Instruction));
    }

    // // Verify EVM address
    if &secp256k1_instruction_layout.eth_address != signer.as_ref() {
        return Err(error!(ErrorCode::InvalidEvmAddress));
    }

    // Verify message
    if &secp256k1_instruction.data[
        secp256k1_instruction_layout.message_data_offset as usize
            ..(secp256k1_instruction_layout.message_data_offset + secp256k1_instruction_layout.message_data_size) as usize
        ] != message {
        return Err(error!(ErrorCode::InvalidMessage));
    }

    // Verify signature
    if &secp256k1_instruction_layout.signature != signature.as_ref() {
        return Err(error!(ErrorCode::InvalidSignature));
    }

    Ok(())
}

pub fn format_message(
    kv_pairs: &Vec<KeyValuePair>,
) -> Result<Vec<u8>> {
    let mut encoded_message = Vec::new();
    encoded_message.extend_from_slice(b"\x19\x01");
    encoded_message.extend_from_slice(&domain_separator());
    encoded_message.extend_from_slice(&hash_signature_data(kv_pairs));
    Ok(encoded_message)
}

fn domain_separator() -> [u8; 32] {
    const NAME: &[u8; 22] = b"CyberAgreementRegistry";
    const VERSION: &[u8; 1] = b"1";
    const CHAIN_ID: u64 = 1;
    const VERIFYING_CONTRACT: [u8;20] = hex!("a9E808B8eCBB60Bb19abF026B5b863215BC4c134");

    let type_hash = keccak::hash(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
    let name_hash = keccak::hash(NAME);
    let version_hash = keccak::hash(VERSION);

    // Convert chain_id to 32-byte big-endian (padded for ABI compliance)
    let mut chain_id_padded = [0u8; 32];
    chain_id_padded[24..].copy_from_slice(&CHAIN_ID.to_be_bytes());

    // Pad verifying_contract (20-byte address) to 32 bytes (left-padded with zeros)
    let mut verifying_contract_padded = [0u8; 32];
    verifying_contract_padded[12..].copy_from_slice(&VERIFYING_CONTRACT);

    keccak::hashv(&[
        &type_hash.to_bytes(),
        &name_hash.to_bytes(),
        &version_hash.to_bytes(),
        &chain_id_padded,
        &verifying_contract_padded,
    ]).to_bytes()
}

fn hash_signature_data(
    kv_pairs: &Vec<KeyValuePair>
) -> [u8; 32] {
    keccak::hashv(&[
        &keccak::hash(b"SignatureData(KeyValuePair[] kvPairs)KeyValuePair(string key,string value)").to_bytes(),
        &hash_key_value_pairs(kv_pairs),
    ]).to_bytes()
}

fn hash_key_value_pairs(
    kv_pairs: &Vec<KeyValuePair>
) -> [u8; 32] {
    let slices: Vec<[u8; 32]> = kv_pairs.iter()
        .map(|kv| hash_key_value_pair(kv))  // Apply converter to each &i32 (dereference with &input)
        .collect();
    let byte_slices: Vec<&[u8]> = slices.iter().map(|arr| arr.as_ref()).collect();
    keccak::hashv(&byte_slices).to_bytes()
}

fn hash_key_value_pair(
    kv_pair: &KeyValuePair
) -> [u8; 32] {
    keccak::hashv(&[
        &keccak::hash(b"KeyValuePair(string key,string value)").to_bytes(),
        &keccak::hash(kv_pair.key.as_bytes()).to_bytes(),
        &keccak::hash(kv_pair.value.as_bytes()).to_bytes(),
    ]).to_bytes()
}

// https://github.com/solana-foundation/solana-web3.js/blob/7d058578462d4592fa1bcf2c393729d08fa75c02/src/secp256k1-program.ts#L49-L75
#[derive(AnchorSerialize, AnchorDeserialize)]
struct Secp256k1InstructionLayout {
    num_signatures: u8,
    signature_offset: u16,
    signature_instruction_index: u8,
    eth_address_offset: u16,
    eth_address_instruction_index: u8,
    message_data_offset: u16,
    message_data_size: u16,
    message_instruction_index: u8,
    eth_address: [u8;20],
    signature: [u8;64],
    recovery_id: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Missing secp256k1 instruction")]
    MissingSecp256k1Instruction,
    #[msg("Invalid secp256k1 program ID")]
    InvalidSecp256k1Program,
    #[msg("Invalid secp256k1 instruction data")]
    InvalidSecp256k1Instruction,
    #[msg("Invalid EVM address")]
    InvalidEvmAddress,
    #[msg("Invalid message")]
    InvalidMessage,
    #[msg("Invalid signature")]
    InvalidSignature,
}
