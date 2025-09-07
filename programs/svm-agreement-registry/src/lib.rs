use anchor_lang::prelude::*;
use solana_program::sysvar::instructions::{
    ID as SYSVAR_IX_ID,
    load_current_index_checked,
    load_instruction_at_checked,
};
use solana_program::ed25519_program;

declare_id!("VnpPHJweU4dLG6BATCh6MnaS1Y81hTC2esBpxMpDzg6");

#[program]
pub mod svm_agreement_registry {
    use super::*;

    pub fn store_data(
        ctx: Context<StoreData>,
        kv_pairs: Vec<KeyValuePair>,
        signer: Pubkey,
        signature: [u8; 64],
        message: Vec<u8>, // TODO WIP: we are supposed to reconstruct the message on-chain
    ) -> Result<()> {
        msg!("DataEntry::INIT_SPACE: {:?}", DataEntry::INIT_SPACE);
        msg!("store_data from: {:?}", ctx.program_id);
        msg!("Signer public key: {:?}", signer);

        // Ed25519 Program is not invokable by other programs:
        // https://docs.rs/solana-program/2.3.0/solana_program/index.html#native-programs
        // Therefore, we rely on the transaction signer to call it himself
        // immediately before calling this method, and we will verify if he had done so correctly.

        // Verify and extract the prior instruction (presumably an Ed25519Program instruction for signature verification)
        let current_index = load_current_index_checked(&ctx.accounts.sysvar_ix)?;
        if current_index == 0 {
            return Err(error!(ErrorCode::MissingEd25519Instruction));
        }
        let ed25519_instruction = load_instruction_at_checked((current_index - 1) as usize, &ctx.accounts.sysvar_ix)?;

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

        let data_entry = &mut ctx.accounts.data_entry;
        data_entry.kv_pairs = kv_pairs;
        data_entry.signer = signer;
        data_entry.signature = signature;

        msg!("Agreement data stored!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct StoreData<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + DataEntry::INIT_SPACE
    )]
    pub data_entry: Account<'info, DataEntry>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Manually verify `sysvar_ix` provided is indeed the SysvarInstructions, as it is not
    /// built-in in Anchor.
    #[account(address = SYSVAR_IX_ID)]
    pub sysvar_ix: AccountInfo<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct DataEntry {
    #[max_len(25)]
    pub kv_pairs: Vec<KeyValuePair>,
    pub signer: Pubkey,
    pub signature: [u8; 64],
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
