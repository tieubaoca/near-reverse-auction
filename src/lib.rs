use near_sdk::*;
use near_sdk::collections::*;
use near_sdk::json_types::{U128, ValidAccountId,Base64VecU8};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise, StorageUsage};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize,Serialize};
use std::collections::HashMap;
use timer::Timer;
const MINT_FEE:Balance=1_000_000_000_000_000_000_000_000;
const CREATE_AUCTION_FEE:Balance=5_000_000_000_000_000_000;
const AUCTION_FEE :f32 = 0.0005;
type TokenId = u32;
type AuctionId = u32;
type Price = Balance;
#[near_bindgen]
#[derive(BorshSerialize,BorshDeserialize,PanicOnDefault)]
pub struct Contract{
    pub owner_id:AccountId,
    pub token_id:TokenId,
    pub auction_id:AuctionId,
    pub auction_by_id:UnorderedMap<AuctionId,Auction>,
    pub auction_id_by_owner: UnorderedMap<AccountId,Vec<AuctionId>>,
    pub token_id_auctioned: Vec<TokenId>,
    pub token_by_owner: UnorderedMap<AccountId,Vec<Token>>,
    pub token_by_id: UnorderedMap<TokenId,Token>,
    pub tokendata_by_id: UnorderedMap<TokenId,TokenData>
}

#[near_bindgen]
impl Contract{
    #[init]
    pub fn new(owner_id:AccountId)->Self{
        Self{
            owner_id,
            token_id:0,
            auction_id:0,
            auction_by_id:UnorderedMap::new(b"auction_by_id".to_vec()),
            auction_id_by_owner:UnorderedMap::new(b"auction_id_by_owner".to_vec()),
            token_by_owner:UnorderedMap::new(b"token_by_owner".to_vec()),
            token_id_auctioned:Vec::new(),
            token_by_id: UnorderedMap::new(b"token_by_id".to_vec()),
            tokendata_by_id:UnorderedMap::new(b"tokendata_by_id".to_vec())
        }
    }
    pub fn get_token_by_owner(&self,_owner_id:AccountId) -> Vec<Token> {
        self.token_by_owner.get(&_owner_id).unwrap()
    }
    #[payable]
    pub fn mint_nft(&mut self, _owner_id:AccountId, _token_data: TokenData)
    {
        
        assert_eq!(env::attached_deposit(), MINT_FEE, "deposit < price");
        // let hash_vec: Vec<u8>=_token_data.clone().secret_hash.unwrap().into();
        // assert_eq!(hash_vec,env::sha256(&_token_data.clone().secret.unwrap().as_bytes()),"Hash is not equal");
        
        self.token_id+=1;
        let  token = Token{
            owner_id:_owner_id.clone(),
            token_id:self.token_id,
            tokendata:_token_data.clone(),
        };
        self.token_by_id.insert(&self.token_id, &token);
        self.tokendata_by_id.insert(&self.token_id,&_token_data);
        self.add_owner_id_token(&_owner_id,&token);

    }
    pub fn transfer_nft(&mut self,_owner_id:ValidAccountId,_token_id:TokenId){
        assert_eq!(self.get_token_by_id(_token_id).owner_id,env::predecessor_account_id(),"You Do Not Own This Token");
        self.token_by_id.get(&_token_id).unwrap().transfer(_owner_id.clone());
        self.add_owner_id_token(&_owner_id.into(),&self.token_by_id.get(&_token_id).unwrap())
    }
    pub fn get_token_by_id(&self,_token_id:TokenId)->Token{
        self.token_by_id.get(&_token_id).unwrap()
    }
    #[payable]
    pub fn create_auction (&mut self,_token_id: TokenId,_starting_price:u64,_auction_time:u64) ->Auction {
        assert_eq!(env::attached_deposit(), CREATE_AUCTION_FEE,"Need 5 N To Create An Auction");
        assert_eq!(env::predecessor_account_id(),self.token_by_id.get(&_token_id).unwrap().owner_id,"You Do Not Own This Token!");
        assert_eq!(self.is_token_autioned(&_token_id),false,"This Token Already In A Auction");
        env::log(env::predecessor_account_id().as_bytes());
        env::log(self.token_by_id.get(&_token_id).unwrap().owner_id.as_bytes());

        self.auction_id+=1;
        let auction = Auction{
            owner_id:env::predecessor_account_id(), 
            auction_id:self.auction_id,
            id_token_auction:_token_id.clone(), 
            auction_time_second:_auction_time,
            starting_price:_starting_price.clone(),
            token_auction:self.token_by_id.get(&_token_id).unwrap(),
            is_enabled:false,
            is_end:false,
            participants: HashMap::new(),
        };
        self.auction_by_id.insert(&self.auction_id,&auction);
        self.token_id_auctioned.push(_token_id);
        auction
    }
    pub fn close_auction(&mut self,_aution_id:AuctionId){
        self.auction_by_id.get(&_aution_id).unwrap().is_enabled=false;
        self.auction_by_id.get(&_aution_id).unwrap().is_end=true;
    }
    fn is_auctione_owner_exist(&self, _owner_id:AccountId) -> bool{
        for owner_id in self.auction_id_by_owner.keys()
        {
            if owner_id==_owner_id {return true;}
        }
        return false;
    }
    fn is_token_owner_exist(&mut self,_owner_id:AccountId) -> bool {
        for owner_id in self.token_by_owner.keys()
        {
            if owner_id==_owner_id {return true;}
        }
        return false;
    }
    fn is_token_autioned(&self,_token_id: &TokenId) -> bool {
        for token_id in &self.token_id_auctioned {
            if token_id == _token_id {return true;}
        }
        return false;
    }
    fn add_owner_id_token(&mut self,_owner_id:&AccountId,token:&Token){

        if self.is_token_owner_exist(_owner_id.clone())==true {
            let mut tokens_by_owner = self.token_by_owner.get(&_owner_id).unwrap();
                tokens_by_owner.push(token.clone());
                self.token_by_owner.insert(&_owner_id, &tokens_by_owner);
        }
        else {
            let mut tokens_by_owner :Vec<Token>;
            tokens_by_owner= Vec::new();
            tokens_by_owner.push(token.clone());
            self.token_by_owner.insert(&_owner_id, &tokens_by_owner);
        }
    }
    fn add_owner_id_auction(&mut self,_owner_id:AccountId,_aution_id:AuctionId){

    }
    
}


#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token{
    owner_id:AccountId,
    token_id:TokenId,
    tokendata:TokenData
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
#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction{
    owner_id:AccountId,
    auction_id:AuctionId,
    auction_time_second:u64,
    id_token_auction: TokenId,
    starting_price: u64,
    token_auction: Token,
    is_enabled: bool,
    is_end:bool,
    participants: HashMap<AccountId,Price>
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
