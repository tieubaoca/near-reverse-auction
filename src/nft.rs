use near_sdk::json_types::{ValidAccountId,Base64VecU8};
use near_sdk::{AccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize,Serialize};
type TokenId = u32;
#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token{
    pub owner_id:AccountId,
    pub authorized_id:AccountId,
    pub token_id:TokenId,
    pub tokendata:TokenData
}
impl Token {
    pub fn transfer(&mut self,_new_owner_id:ValidAccountId){
        self.owner_id = _new_owner_id.into();
    }
    
}
#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenData{
    pub title: Option<String>,
    pub description: Option<String>,
    secret: Option<String>,
    pub secret_hash: Option<Base64VecU8>,
}