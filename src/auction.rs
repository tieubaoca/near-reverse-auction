use near_sdk::{AccountId, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize,Serialize};
use std::collections::HashMap;
use std::vec::Vec;
use crate::*;
type TokenId = u32;
type AuctionId = u32;
pub type Price = Balance;
#[derive(Debug,BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction{
    pub owner_id:AccountId,
    pub auction_id:AuctionId,
    pub auction_start_time: u64,
    pub auction_during_second:u64,
    pub id_token_auction: TokenId,
    pub token_auction: Token,
    pub is_enabled: bool,
    pub is_end:bool,
    pub participants: HashMap<AccountId,Price>, 
    pub winner: AccountId,
    pub close_price: Balance
}
impl Auction {
    pub fn calculate_the_single_lowest(&mut self) -> Price{
        let  mut price:Price = 0;
        let mut prices:Vec<&Price> = self.participants.values().collect();
        prices.sort();
        if prices.len()==1
            {
                price = prices[0].clone();
            }
        else if prices[0]!=prices[1] 
            {
                price = prices[0].clone();
            }
        else
            {
                for i in 1..(prices.len() -1)  
                    {
                        if price ==0 
                        {
                        if i< (prices.len() -1)
                            {
                                if prices[i] != prices[i-1] && prices[i] != prices[i+1] 
                                {
                                    price = prices[i].clone();
                                }
                            }
                        else if i== prices.len()
                            {
                                if prices[i] != prices[i-1] 
                                    {
                                        price=prices[i].clone();
                                    }
                            }
                        else 
                            {
                                price = 0;
                            }
                        }
                    }
            }
        price
    }
    pub fn find_winner(&mut self,price:Price){
        let winner = self.participants.iter().find_map(|(key, &val)| if val == price { Some(key.clone()) } else { None }).unwrap();
        self.winner = winner;
        
    }
}