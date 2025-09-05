use anchor_lang::prelude::*;

declare_id!("VnpPHJweU4dLG6BATCh6MnaS1Y81hTC2esBpxMpDzg6");

#[program]
pub mod svm_agreement_registry {
    use super::*;

    pub fn store_data(ctx: Context<StoreData>, kv_pairs: Vec<KeyValuePair>, signature: [u8; 64]) -> Result<()> {
        msg!("store_data from: {:?}", ctx.program_id);

        let data_entry = &mut ctx.accounts.data_entry;
        data_entry.signature = signature;

        // let data_entry = &mut ctx.accounts.data_entry;
        // data_entry.signer = ctx.accounts.signer.key();
        // data_entry.kv_pairs = kv_pairs;
        // data_entry.signature = signature;
        
        Ok(())
    }
}

#[derive(Accounts)]
// #[instruction(kv_pairs: Vec<(String, String)>)]
pub struct StoreData<'info> {
    // #[account(
    //     init_if_needed,
    //     payer = signer,
    //     space = 8 + 32 + 64 + 4 + kv_pairs.len() * (4 + 32 + 4 + 32), // Discriminator + signer + sig + vec len + per pair (str len + data + str len + data); assuming max str len 32
    //     seeds = [b"data", signer.key().as_ref()],
    //     bump
    // )]
    // pub data_entry: Account<'info, DataEntry>,
    // #[account(mut)]
    // pub signer: Signer<'info>,
    // pub system_program: Program<'info, System>,

    #[account(
        init,
        payer = signer,
        space = 8 + 64
    )]
    pub data_entry: Account<'info, DataEntry>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct DataEntry {
    pub signature: [u8; 64],
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

// #[account]
// pub struct DataEntry {
//     pub signer: Pubkey,
//     pub kv_pairs: Vec<(String, String)>,
//     pub signature: [u8; 64],
// }
