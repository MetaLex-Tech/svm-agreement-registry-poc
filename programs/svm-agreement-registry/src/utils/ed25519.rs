use anchor_lang::prelude::*;
use solana_program::sysvar::instructions::{
    ID as SYSVAR_IX_ID,
    load_current_index_checked,
    load_instruction_at_checked,
};
use solana_program::ed25519_program;

pub fn verify_signature(
    ix_sysvar_account_info: &AccountInfo,
    signer: Pubkey,
    signature: [u8; 64],
    message: Vec<u8>,
) -> Result<()> {
    // Verify and extract the prior instruction (presumably an Ed25519Program instruction for signature verification)
    let current_index = load_current_index_checked(&ix_sysvar_account_info)?;
    if current_index == 0 {
        return Err(error!(ErrorCode::MissingEd25519Instruction));
    }
    let ed25519_instruction = load_instruction_at_checked((current_index - 1) as usize, &ix_sysvar_account_info)?;

    // Verify it is a valid Ed25519Program instruction
    if ed25519_instruction.program_id != ed25519_program::id() {
        return Err(error!(ErrorCode::InvalidEd25519Program));
    }

    let ed25519_instruction_layout = Ed25519InstructionLayout::try_from_slice(&ed25519_instruction.data[0..16])?;

    // Verify number of signatures
    if ed25519_instruction_layout.num_signatures != 1 {
        return Err(error!(ErrorCode::InvalidEd25519Instruction));
    }

    // Verify public key
    if &ed25519_instruction.data[
        ed25519_instruction_layout.public_key_offset as usize
            ..(ed25519_instruction_layout.public_key_offset + 32) as usize
        ] != signer.as_ref() {
        return Err(error!(ErrorCode::InvalidPublicKey));
    }

    // Verify message
    if &ed25519_instruction.data[
        ed25519_instruction_layout.message_data_offset as usize
            ..(ed25519_instruction_layout.message_data_offset + ed25519_instruction_layout.message_data_size) as usize
        ] != message {
        return Err(error!(ErrorCode::InvalidMessage));
    }

    // Verify signature
    if &ed25519_instruction.data[
        ed25519_instruction_layout.signature_offset as usize
            ..(ed25519_instruction_layout.signature_offset + 64) as usize
        ] != signature {
        return Err(error!(ErrorCode::InvalidSignature));
    }

    Ok(())
}

// TODO add the missing fields once solana-cli has finally followed the spec (PR: https://github.com/anza-xyz/agave/issues/3340)
pub fn format_message(
    kv_pairs: &Vec<KeyValuePair>,
) -> Result<Vec<u8>> {
    let mut data = b"\xffsolana offchain".to_vec(); // signing domain
    data.push(0); // header version TODO WIP: hard-coded as 0 for now
    data.push(0); // message format TODO WIP: hard-coded as 0 for now

    let mut serialized_message = Vec::new();
    kv_pairs.serialize(&mut serialized_message)?;

    data.extend_from_slice(&(serialized_message.len() as u16).to_le_bytes()); // message length
    data.extend_from_slice(&serialized_message); // message body
    Ok(data)
}

#[derive(InitSpace, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct KeyValuePair {
    #[max_len(50)]
    pub key: String,
    #[max_len(50)]
    pub value: String,
}

// https://github.com/solana-foundation/solana-web3.js/blob/7d058578462d4592fa1bcf2c393729d08fa75c02/src/ed25519-program.ts#L33-L55
#[derive(AnchorSerialize, AnchorDeserialize)]
struct Ed25519InstructionLayout {
    num_signatures: u8,
    padding: u8,
    signature_offset: u16,
    signature_instruction_index: u16,
    public_key_offset: u16,
    public_key_instruction_index: u16,
    message_data_offset: u16,
    message_data_size: u16,
    message_instruction_index: u16,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Missing Ed25519 instruction")]
    MissingEd25519Instruction,
    #[msg("Invalid Ed25519 program ID")]
    InvalidEd25519Program,
    #[msg("Invalid Ed25519 instruction data")]
    InvalidEd25519Instruction,
    #[msg("Invalid public key")]
    InvalidPublicKey,
    #[msg("Invalid message")]
    InvalidMessage,
    #[msg("Invalid signature")]
    InvalidSignature,
}
