use anchor_lang::prelude::*;

declare_id!("VnpPHJweU4dLG6BATCh6MnaS1Y81hTC2esBpxMpDzg6");

#[program]
pub mod svm_agreement_registry {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
