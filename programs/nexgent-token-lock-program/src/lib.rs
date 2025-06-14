use anchor_lang::prelude::*;

declare_id!("HWZXK9n72Hj5kyadBph9PFbWcWq3RsEXFTEd2uFEDDK8");

#[program]
pub mod nexgent_token_lock_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
