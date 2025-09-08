use anchor_lang::prelude::*;
use solana_program::sysvar::instructions::{
    ID as SYSVAR_IX_ID,
    load_current_index_checked,
    load_instruction_at_checked,
};
use solana_program::secp256k1_program;

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
