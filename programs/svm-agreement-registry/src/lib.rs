use anchor_lang::prelude::*;

declare_id!("VnpPHJweU4dLG6BATCh6MnaS1Y81hTC2esBpxMpDzg6");

#[program]
pub mod svm_agreement_registry {
    use super::*;

    pub fn store_data(ctx: Context<StoreData>, kv_pairs: Vec<KeyValuePair>, signature: [u8; 64]) -> Result<()> {
        msg!("DataEntry::INIT_SPACE: {:?}", DataEntry::INIT_SPACE);
        msg!("store_data from: {:?}", ctx.program_id);

        let data_entry = &mut ctx.accounts.data_entry;
        data_entry.kv_pairs = kv_pairs;
        data_entry.signature = signature;

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
}

#[account]
#[derive(InitSpace)]
pub struct DataEntry {
    #[max_len(25)]
    pub kv_pairs: Vec<KeyValuePair>,
    pub signature: [u8; 64],
}

#[derive(InitSpace, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct KeyValuePair {
    #[max_len(50)]
    pub key: String,
    #[max_len(50)]
    pub value: String,
}
