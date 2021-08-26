use crate::*;
use near_sdk::json_types::ValidAccountId;
use non_fungible_token::TokenId;

#[ext_contract(ext_other)]
pub trait OtherContract {
    fn nft_transfer(
        &mut self,
        receiver_id: ValidAccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: ValidAccountId,
        msg: Option<String>,
    ) -> Option<Promise>;
}