use anchor_lang::prelude::*;

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
