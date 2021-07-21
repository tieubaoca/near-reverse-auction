use near_sdk::*;
use near_sdk::collections::*;
use near_sdk::json_types::{ValidAccountId,Base64VecU8};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize,Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
const TRANSFER_FEE:Balance=1_000_000_000_000_000_000_000;
const MINT_FEE:Balance=1_000_000_000_000_000_000_000_000;
const CREATE_AUCTION_FEE:Balance=5_000_000_000_000_000_000_000_000;
const AUCTION_FEE :Balance = 5;
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
    pub auction_going_on: Vec<AuctionId>,
    pub token_id_auctioned: Vec<TokenId>,
    pub token_by_owner: UnorderedMap<AccountId,Vec<TokenId>>,
    pub token_by_id: UnorderedMap<TokenId,Token>
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
            auction_going_on: Vec::new(),
            token_by_owner:UnorderedMap::new(b"token_by_owner".to_vec()),
            token_id_auctioned:Vec::new(),
            token_by_id: UnorderedMap::new(b"token_by_id".to_vec())
        }
    }
    pub fn get_token_by_owner(&self,_owner_id:AccountId) -> Vec<Token> {
        let token_id = self.token_by_owner.get(&_owner_id).unwrap();
        let mut tokens:Vec<Token> = Vec::new();
        for id in token_id {
            tokens.push(self.token_by_id.get(&id).unwrap());
        }
        tokens
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
            authorized_id: AccountId::new(),
            token_id:self.token_id,
            tokendata:_token_data.clone(),
        };
        self.token_by_id.insert(&self.token_id, &token);
        self.add_owner_id_token(&_owner_id,&token.token_id);

    }
    pub fn transfer_nft(&mut self,_new_owner_id:ValidAccountId,_token_id:TokenId){
        let sender = env::predecessor_account_id();
        if sender != self.owner_id && sender != self.token_by_id.get(&_token_id).unwrap().owner_id {
            env::panic("You Can Not Transfer This NFT".as_bytes());
        }
        let mut token =  self.token_by_id.get(&_token_id).unwrap();
        token.transfer(_new_owner_id.clone());
        self.token_by_id.insert(&_token_id,&token);
        self.add_owner_id_token(&_new_owner_id.into(),&_token_id)
    }
    pub fn get_token_by_id(&self,_token_id:TokenId)->Token{
        self.token_by_id.get(&_token_id).unwrap()
    }
    #[payable]
    pub fn create_auction (&mut self,_token_id: TokenId,_auction_during_seconds:u64) ->Auction {
        assert_eq!(env::attached_deposit(), CREATE_AUCTION_FEE,"Need 5 N To Create An Auction");
        assert_eq!(env::predecessor_account_id(),self.token_by_id.get(&_token_id).unwrap().owner_id,"You Do Not Own This Token!");
        assert_eq!(self.is_token_auctioned(&_token_id),false,"This Token Already In An Auction");
        env::log(env::predecessor_account_id().as_bytes());
        env::log(self.token_by_id.get(&_token_id).unwrap().owner_id.as_bytes());
        self.commit_nft(&_token_id);
        self.auction_id+=1;
        let auction = Auction{
            owner_id:env::predecessor_account_id(), 
            auction_id:self.auction_id,
            id_token_auction:_token_id.clone(), 
            auction_start_time:0,
            auction_during_second:_auction_during_seconds,
            token_auction:self.token_by_id.get(&_token_id).unwrap(),
            is_enabled:false,
            is_end:false,
            participants: HashMap::new(),
            winner: AccountId::new()
        };
        self.auction_by_id.insert(&self.auction_id,&auction);
        self.token_id_auctioned.push(_token_id);
        self.add_owner_id_auction(&auction);
        
        auction
    }
    fn commit_nft(&mut self,_token_id:&TokenId){
        let mut token = self.token_by_id.get(&_token_id).unwrap();
        token.authorized_id = self.owner_id.clone();
        self.token_by_id.insert(&self.token_id,&token);
    }
    fn release_nft(&mut self,_token_id:&TokenId){
        let mut token = self.token_by_id.get(&_token_id).unwrap();
        token.authorized_id = AccountId::new();
        self.token_by_id.insert(&self.token_id,&token);
    }
    pub fn start_auction(&mut self,_auction_id:AuctionId){
        assert_eq!(env::predecessor_account_id(),self.auction_by_id.get(&_auction_id).unwrap().owner_id,"You Do Not Own This Auction");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_end,false,"This Auction Already Ends");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_enabled,false,"This Auction Already Begins");
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.auction_start_time = env::block_timestamp();
        auction.is_enabled=true;
        self.auction_by_id.insert(&_auction_id,&auction);
        self.auction_going_on.push(_auction_id);
          
    }
    pub fn get_auctions_by_owner(&self, _owner_id:AccountId) -> Vec<Auction>{
        let mut auctions:Vec<Auction>=Vec::new();
        for id in self.auction_id_by_owner.get(&_owner_id).unwrap(){
            auctions.push(self.auction_by_id.get(&id).unwrap());
        }
        auctions
    }
    #[payable]
    pub fn commit_auction(&mut self, _auction_id:AuctionId){
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_end,false,"This Auction Alredy Ends");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_enabled,true,"This Auction Does Not Begin");
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.participants.insert(env::predecessor_account_id(),env::attached_deposit());
        self.auction_by_id.insert(&_auction_id, &auction);
    }
    #[private]
    pub fn check_auctions(&mut self){
        let list_auction_id = self.auction_going_on.clone();
        for item in   list_auction_id {
            let endtime:u64= self.auction_by_id.get(&item).unwrap().auction_start_time+self.auction_by_id.get(&item).unwrap().auction_during_second*1_000_000_000;
            env::log(endtime.to_string().as_bytes());
            env::log(item.to_string().as_bytes());
            let current_time = env::block_timestamp();
            if endtime <= current_time
            {
                
                let price = self.auction_by_id.get(&item).unwrap().calculate_the_single_lowest();
                env::log(price.to_string().as_bytes());
                if price != 0 as Balance {
                    let mut auction = self.auction_by_id.get(&item).unwrap();
                    auction.find_winner(price);
                    
                    auction.participants.remove(&auction.winner);
                    self.auction_by_id.insert(&item,&auction);
                    self.transfer_nft(ValidAccountId::try_from(auction.winner).unwrap(),auction.id_token_auction);
                    self.transfer_ft_to_seller(ValidAccountId::try_from(auction.owner_id).unwrap(),price);
                }
                
                
                self.transfer_ft_back_to_participants(self.auction_by_id.get(&item).unwrap().participants);
                
                self.close_auction(item.clone());
            }
        }
    }
    fn transfer_ft_back_to_participants(&self,_participants: HashMap<AccountId,Balance>){
        for (account,balance) in _participants.iter(){
            let account = Promise::new(account.clone());
            account.transfer(balance-TRANSFER_FEE);
        }

    }
    fn transfer_ft_to_seller(&self,_owner_id:ValidAccountId,_auction_price:Balance){
        let account = Promise::new(_owner_id.into());
        account.transfer(_auction_price*(1000 -AUCTION_FEE)/1000);
    }
    pub fn close_auction(&mut self,_auction_id:AuctionId){
        if env::predecessor_account_id()!=self.owner_id&&env::predecessor_account_id()!=self.auction_by_id.get(&_auction_id).unwrap().owner_id{
            env::panic("You Can Not Close This Auction".as_bytes());
        }
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.is_enabled=false;
        auction.is_end=true;
        self.auction_by_id.insert(&_auction_id, &auction);
        let _token_id = self.auction_by_id.get(&_auction_id).unwrap().id_token_auction;
        self.release_nft(&_token_id);
        self.token_id_auctioned.retain(|&x| x != _token_id );
        self.auction_going_on.retain(|&x| x != _auction_id);
    }
    fn is_auction_owner_exist(&self, _owner_id:AccountId) -> bool{
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
    fn is_token_auctioned(&self,_token_id: &TokenId) -> bool {
        for token_id in &self.token_id_auctioned {
            if token_id == _token_id {return true;}
        }
        return false;
    }
    fn add_owner_id_token(&mut self,_owner_id:&AccountId,token:&TokenId){

        if self.is_token_owner_exist(_owner_id.clone()) {
            let mut tokens_id = self.token_by_owner.get(&_owner_id).unwrap();
            tokens_id.push(token.clone());
            self.token_by_owner.insert(&_owner_id,&tokens_id);
        }
        else {
            let mut tokens_by_owner :Vec<TokenId>;
            tokens_by_owner= Vec::new();
            tokens_by_owner.push(token.clone());
            self.token_by_owner.insert(&_owner_id, &tokens_by_owner);
        }
    }
    fn add_owner_id_auction(&mut self,_auction:&Auction){
        if self.is_auction_owner_exist(_auction.clone().owner_id) {
            let mut auctions = self.auction_id_by_owner.get(&_auction.owner_id).unwrap();
            auctions.push(_auction.auction_id);
            self.auction_id_by_owner.insert(&_auction.owner_id,&auctions);
        }
        else {
            let mut auctions_by_owner :Vec<AuctionId> = Vec::new();
            auctions_by_owner.push(_auction.auction_id);
            self.auction_id_by_owner.insert(&_auction.owner_id, &auctions_by_owner);
        }
    }
    
}


#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token{
    owner_id:AccountId,
    authorized_id:AccountId,
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
#[derive(BorshDeserialize,BorshSerialize,Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction{
    owner_id:AccountId,
    auction_id:AuctionId,
    auction_start_time: u64,
    auction_during_second:u64,
    id_token_auction: TokenId,
    token_auction: Token,
    is_enabled: bool,
    is_end:bool,
    participants: HashMap<AccountId,Price>, 
    winner: AccountId
}
impl Auction {
    pub fn calculate_the_single_lowest(&self) -> Price{
        
        let mut prices:Vec<&Price> = self.participants.values().collect();
        prices.sort();
        if prices.len()==1 {return prices[0].clone();}
        else if prices[0]!=prices[1] {return prices[0].clone();}
        for i in 1..(prices.len() -2)  {
             if prices[i]!=prices[i-1]&&prices[i]!=prices[i+1] {return prices[i].clone();}
        }
        if prices[prices.len()]!=prices[prices.len()-1] {return prices[prices.len()].clone();}
        else {return 0;}
    }
    pub fn find_winner(&mut self,price:Price){
        let winner = self.participants.iter().find_map(|(key, &val)| if val == price { Some(key.clone()) } else { None }).unwrap();
        self.winner = winner;
        
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
