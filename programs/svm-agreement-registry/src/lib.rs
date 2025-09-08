use anchor_lang::prelude::*;
use solana_program::sysvar::instructions::ID as SYSVAR_IX_ID;

pub mod utils;

declare_id!("VnpPHJweU4dLG6BATCh6MnaS1Y81hTC2esBpxMpDzg6");

#[program]
pub mod svm_agreement_registry {
    use super::*;

    pub fn propose_and_sign_agreement(
        ctx: Context<StoreData>,
        kv_pairs: Vec<utils::offchain_message::KeyValuePair>,
        signer: Pubkey,
        signature: [u8; 64],
    ) -> Result<()> {
        // Ed25519 Program is not invokable by other programs:
        // https://docs.rs/solana-program/2.3.0/solana_program/index.html#native-programs
        // Therefore, we rely on the transaction signer to call it himself
        // immediately before calling this method, and we will verify if he had done so correctly.
        utils::ed25519::verify_signature(
            &ctx.accounts.sysvar_ix,
            signer,
            signature,
            // Reconstruct the message from key-value pairs
            utils::offchain_message::format_message(&kv_pairs)?,
        )?;

        let data_entry = &mut ctx.accounts.data_entry;
        data_entry.kv_pairs = kv_pairs;
        data_entry.signer = signer;
        data_entry.signature = signature;

        msg!("Agreement data stored!");
        Ok(())
    }

    pub fn propose_and_sign_agreement_eth(
        ctx: Context<StoreData>,
        kv_pairs: Vec<utils::offchain_message::KeyValuePair>,
        signer: [u8; 20],
        signature: [u8; 64],
        message: Vec<u8>,
        recovery_id: u8,
    ) -> Result<()> {
        msg!("DataEntry::INIT_SPACE: {:?}", DataEntry::INIT_SPACE);
        msg!("store_data from: {:?}", ctx.program_id);
        msg!("Signer EVM address: {:?}", signer);

        // Secp256k1 Program is not invokable by other programs:
        // https://docs.rs/solana-program/2.3.0/solana_program/index.html#native-programs
        // Therefore, we rely on the transaction signer to call it himself
        // immediately before calling this method, and we will verify if he had done so correctly.
        utils::secp256k1::verify_signature(
            &ctx.accounts.sysvar_ix,
            signer,
            signature,
            recovery_id,
            message, // TODO WIP: reconstruct the message from key-value pairs
        )?;

        let data_entry = &mut ctx.accounts.data_entry;
        data_entry.kv_pairs = kv_pairs;

        // Fit EVM address
        let mut signerAsPubkey = [0u8; 32];
        signerAsPubkey[12..32].copy_from_slice(&signer);

        data_entry.signer = Pubkey::new_from_array(signerAsPubkey);
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
    pub kv_pairs: Vec<utils::offchain_message::KeyValuePair>,
    pub signer: Pubkey,
    pub signature: [u8; 64],
}
