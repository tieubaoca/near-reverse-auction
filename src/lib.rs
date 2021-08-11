use near_sdk::*;
use near_sdk::collections::*;
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
const TRANSFER_FEE:Balance=1_000_000_000_000_000_000_000;
const MINT_FEE:Balance=1_000_000_000_000_000_000_000_000;
const CREATE_AUCTION_FEE:Balance=5_000_000_000_000_000_000_000_000;
use auction::Auction;
use std::convert::TryFrom;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::json_types::ValidAccountId;
mod auction;
type AuctionId = u32;
#[near_bindgen]
#[derive(BorshSerialize,BorshDeserialize,PanicOnDefault)]
pub struct Contract{
    pub owner_id:ValidAccountId,
    pub nft_contract: NonFungibleToken,
    pub auction_id:AuctionId,
    pub auction_by_id:UnorderedMap<AuctionId,Auction>,
    pub auction_id_by_owner: UnorderedMap<AccountId,Vec<AuctionId>>,
    pub auction_going_on: Vec<AuctionId>,
    pub token_id_auctioned: Vec<TokenId>,
    
}

near_contract_standards::impl_non_fungible_token_core!(Contract, nft_contract);
near_contract_standards::impl_non_fungible_token_approval!(Contract, nft_contract);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, nft_contract);

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract{
    #[init]
    pub fn new(_owner_id:ValidAccountId)->Self{
        Self{
            owner_id:_owner_id.clone(),
            nft_contract: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                _owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            auction_id:0,
            auction_by_id:UnorderedMap::new(b"auction_by_id".to_vec()),
            auction_id_by_owner:UnorderedMap::new(b"auction_id_by_owner".to_vec()),
            auction_going_on: Vec::new(),
            token_id_auctioned:Vec::new()
        }
    }
    pub fn get_token_by_owner(&self,_owner_id:AccountId) -> UnorderedSet<TokenId> {
        
        self.nft_contract.tokens_per_owner.as_ref().unwrap().get(&_owner_id).unwrap()
    }
    #[payable]
    pub fn mint_nft(&mut self, _owner_id:ValidAccountId, _token_id: String, _token_data: TokenMetadata) -> Token
    {
        
        assert_eq!(env::attached_deposit(), MINT_FEE, "deposit != MINT_FEE");
        // let hash_vec: Vec<u8>=_token_data.clone().secret_hash.unwrap().into();
        // assert_eq!(hash_vec,env::sha256(&_token_data.clone().secret.unwrap().as_bytes()),"Hash is not equal");
        
        self.nft_contract.mint(_token_id, _owner_id, Some(_token_data)) 

    }
    pub fn transfer_nft(&mut self, _new_owner_id: ValidAccountId, _token_id: TokenId){
       self.nft_contract.nft_transfer(_new_owner_id, _token_id, Some(0), Some(String::new()));
    }
    pub fn get_token_by_id(&self,_token_id:TokenId) -> TokenMetadata {
        self.nft_contract.token_metadata_by_id.as_ref().unwrap().get(&_token_id).unwrap()
    }
    #[payable]
    pub fn create_auction (&mut self,_token_id: TokenId,_auction_during_seconds:u64) ->Auction {
        assert_eq!(env::attached_deposit(), CREATE_AUCTION_FEE,"Need 5 N To Create An Auction");
        assert!(!self.get_token_by_owner(env::predecessor_account_id()).contains(&_token_id),"You Do Not Own This Token!");
        assert_eq!(self.is_token_auctioned(&_token_id),false,"This Token Already In An Auction");
        self.nft_contract.nft_approve(_token_id.clone(),self.owner_id.clone(),Some(String::new()));
        self.auction_id+=1;
        let auction = Auction{
            owner_id:ValidAccountId::try_from(env::predecessor_account_id()).unwrap(), 
            auction_id:self.auction_id,
            id_token_auction:_token_id.clone(), 
            auction_start_time:0,
            auction_during_second:_auction_during_seconds,
            is_enabled:false,
            is_end:false,
            participants: UnorderedMap::new(b"paticipants".to_vec()),
            winner: AccountId::new(), 
            close_price: 0
        };
        self.auction_by_id.insert(&self.auction_id,&auction);
        self.token_id_auctioned.push(_token_id);
        self.add_owner_id_auction(&auction);
        
        auction
    }
    pub fn start_auction(&mut self,_auction_id:AuctionId){
        assert_eq!(env::predecessor_account_id(),self.auction_by_id.get(&_auction_id).unwrap().owner_id.to_string(),"You Do Not Own This Auction");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_end,false,"This Auction Already Ends");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_enabled,false,"This Auction Already Begins");
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.auction_start_time = env::block_timestamp();
        auction.is_enabled=true;
        self.auction_by_id.insert(&_auction_id,&auction);
        self.auction_going_on.push(_auction_id);
          
    }
    pub fn get_auctions_by_owner(&self, _owner_id:AccountId) -> Vec<Auction>{
        let mut auctions:Vec<Auction> =Vec::new();
        for id in self.auction_id_by_owner.get(&_owner_id).unwrap(){
            let mut auction = self.auction_by_id.get(&id).unwrap();
            auction.participants =  UnorderedMap::new(b"paticipants".to_vec());
            auctions.push(auction);
        }
        auctions
    }
    pub fn get_auction_by_id(&self,_auction_id:AuctionId) -> Auction{
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.participants =  UnorderedMap::new(b"paticipants".to_vec());
        auction
    }
    #[payable]
    pub fn commit_auction(&mut self, _auction_id:AuctionId){
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        assert!(env::attached_deposit()>0,"You Can Not Commit 0N");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_end,false,"This Auction Alredy Ends");
        assert_eq!(self.auction_by_id.get(&_auction_id).unwrap().is_enabled,true,"This Auction Does Not Begin");
        assert!(!auction.participants.keys().any(|x| x.to_string() == env::predecessor_account_id()),"You Have Already Commited {}",env::predecessor_account_id());
        auction.participants.insert(&ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),&env::attached_deposit());
        self.auction_by_id.insert(&_auction_id, &auction);
    }
    #[private]
    pub fn check_auctions(&mut self){
        let list_auction_id = self.auction_going_on.clone();
        for item in   list_auction_id {
            let endtime:u64= self.auction_by_id.get(&item).unwrap().auction_start_time+self.auction_by_id.get(&item).unwrap().auction_during_second*1_000_000_000;
           
            let current_time = env::block_timestamp();
            if endtime <= current_time
            {
                
                let price = self.auction_by_id.get(&item).unwrap().calculate_the_single_lowest();
                
                if price != 0 as Balance {
                    let mut auction = self.auction_by_id.get(&item).unwrap();
                    auction.close_price = price.clone();
                    auction.find_winner(price);
                    let mut msg = auction.winner.to_string().clone();
                    msg.push_str(&price.to_string());
                    env::log(msg.as_bytes());
                    auction.participants.remove(&ValidAccountId::try_from(auction.winner.as_ref()).unwrap());
                    self.auction_by_id.insert(&item,&auction);
                    self.transfer_nft(ValidAccountId::try_from(auction.winner).unwrap(),auction.id_token_auction);
                    self.transfer_ft_to_seller(auction.owner_id.to_string(),price);
                }
                
                
                self.transfer_ft_back_to_participants(self.auction_by_id.get(&item).unwrap().participants);
                
                self.close_auction(item.clone());
            }
        }
    }
    fn transfer_ft_back_to_participants(&self,_participants: UnorderedMap<ValidAccountId,Balance>){
        for (account,balance) in _participants.iter(){
            let account = Promise::new(account.to_string());
            account.transfer(balance-TRANSFER_FEE);
        }

    }
    fn transfer_ft_to_seller(&self,_owner_id:AccountId,_auction_price:Balance){
        let account = Promise::new(_owner_id.into());
        account.transfer(_auction_price);
    }
    pub fn close_auction(&mut self,_auction_id:AuctionId){
        if env::predecessor_account_id() != self.owner_id.to_string() && 
        env::predecessor_account_id() != self.auction_by_id.get(&_auction_id).unwrap().owner_id.to_string()
        {
            env::panic("You Can Not Close This Auction".as_bytes());
        }
        if self.auction_by_id.get(&_auction_id).unwrap().is_enabled==true && 
            env::predecessor_account_id()!=self.owner_id.to_string() {
            env::panic("You Can Not Close Auction After Start".as_bytes());
        }
        let mut auction = self.auction_by_id.get(&_auction_id).unwrap();
        auction.is_enabled=false;
        auction.is_end=true;
        self.auction_by_id.insert(&_auction_id, &auction);
        let _token_id = self.auction_by_id.get(&_auction_id).unwrap().id_token_auction;
        self.token_id_auctioned.retain(|x| x != &_token_id );
        self.auction_going_on.retain(|&x| x != _auction_id);
    }
    fn is_auction_owner_exist(&self, _owner_id:AccountId) -> bool{
        for owner_id in self.auction_id_by_owner.keys()
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

    fn add_owner_id_auction(&mut self,_auction:&Auction){
        if self.is_auction_owner_exist(_auction.clone().owner_id.to_string()) {
            let mut auctions = self.auction_id_by_owner.get(&_auction.owner_id.to_string()).unwrap();
            auctions.push(_auction.auction_id);
            self.auction_id_by_owner.insert(&_auction.owner_id.to_string(),&auctions);
        }
        else {
            let mut auctions_by_owner :Vec<AuctionId> = Vec::new();
            auctions_by_owner.push(_auction.auction_id);
            self.auction_id_by_owner.insert(&_auction.owner_id.to_string(), &auctions_by_owner);
        }
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::{ValidAccountId, Base64VecU8};
    use near_sdk::Balance;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    fn bob() ->   ValidAccountId {
        ValidAccountId::try_from("bob.testnet").unwrap()
    }
    fn senna() -> ValidAccountId {
        ValidAccountId::try_from("senna.testnet").unwrap()
    }
    fn alice() -> ValidAccountId {
        ValidAccountId::try_from("alice.testnet").unwrap()
    }
    fn carol() -> ValidAccountId {
        ValidAccountId::try_from("carol.testnet").unwrap()
    }
    fn smith() -> ValidAccountId {
        ValidAccountId::try_from("smith.testnet").unwrap()
    }
    fn john() -> ValidAccountId {
        ValidAccountId::try_from("john.testnet").unwrap()
    }
    fn lili() -> ValidAccountId {
        ValidAccountId::try_from("ili.testnet").unwrap()
    }
    fn james() -> ValidAccountId {
        ValidAccountId::try_from("james.testnet").unwrap()
    }
    fn nft(nft: String) -> TokenMetadata {
        TokenMetadata{
            title: Some(nft.clone()), // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: Some(nft.clone()), // free-form description
            media: Some(String::new()), // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: Some(Base64VecU8::from(Vec::new())), // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: Some(10), // number of copies of this set of metadata in existence when token was minted.
            issued_at:  Some(String::new()), // ISO 8601 datetime when token was issued or minted
            expires_at:  Some(String::new()), // ISO 8601 datetime when token expires
            starts_at:  Some(String::new()), // ISO 8601 datetime when token starts being valid
            updated_at: Some(String::new()), // ISO 8601 datetime when token was last updated
            extra: Some(String::new()), // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference:  Some(String::new()), // URL to an off-chain JSON file with more info.
            reference_hash: Some(Base64VecU8::from(Vec::new())), // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
            }
    }

    // part of writing unit tests is setting up a mock context
    // this is a useful list to peek at when wondering what's available in env::*
    fn get_context(_account_id: String, storage_usage: u64, block_timestamp: Timestamp,attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: _account_id.clone(),
            signer_account_id: _account_id.clone(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id:_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp,
            account_balance: 1_00_000_000_000_000_000_000_000_000,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }
    #[test]
    fn test_mint_nft() {
        let context = get_context(senna().to_string(), 0,0,MINT_FEE);
        testing_env!(context);
        let mut contract = Contract::new(senna());
        contract.mint_nft(senna(),String::from("abcnft01"),nft("first".to_string()));
        contract.mint_nft(senna(),String::from("abcnft02"),nft("second".to_string()));
        testing_env!(get_context(bob().to_string(),env::storage_usage(),0,MINT_FEE));
        contract.mint_nft(bob(),String::from("abcnft03"),nft("third".to_string()));
        assert_eq!(contract.nft_contract.tokens_per_owner.as_ref().unwrap().get(&senna().to_string()).unwrap().to_vec(),vec![String::from("abcnft01"),String::from("abcnft02")]);
        //assert_eq!(contract.nft_contract.tokens_per_owner.as_ref().unwrap().get(&bob().to_string()).unwrap().to_vec(),vec![String::from("abcnft03")]);
    }
    // #[test]
    // fn test_transfer_nft(){
    //     let context = get_context(senna(),0, 0,MINT_FEE);
    //     testing_env!(context);
    //     let mut contract = Contract::new(senna());
    //     contract.mint_nft(senna(),nft("first".to_string()));
    //     contract.mint_nft(senna(),nft("second".to_string()));
    //     contract.transfer_nft(bob(),2);
    //     assert_eq!(contract.token_by_owner.get(&bob()).unwrap(),vec![2],"");
    //     assert_eq!(contract.token_by_id.get(&2).unwrap().owner_id,bob(),"");
    // }
    // #[test]
    // fn test_create_close_auction(){
    //     let context = get_context(senna(),0, 0,MINT_FEE);
    //     testing_env!(context);
    //     let mut contract = Contract::new(senna());
    //     contract.mint_nft(bob(),nft("first".to_string()));
    //     contract.mint_nft(bob(),nft("second".to_string()));
    //     testing_env!(get_context(bob(),env::storage_usage(),0,CREATE_AUCTION_FEE));
    //     contract.create_auction(2,60);
    //     assert_eq!(contract.token_id_auctioned,vec![2],"");
    //     assert_eq!(contract.auction_id_by_owner.get(&bob()).unwrap(),vec![1],"");
    //     contract.close_auction(1);
    //     assert_eq!(contract.token_id_auctioned,Vec::<TokenId>::new(),"");
    // }
    // #[test]
    // fn hold_auction(){
    //     let s2ns = 1_000_000_000;
    //     let context = get_context(senna(),0, 0,MINT_FEE);
    //     testing_env!(context);
    //     let mut contract = Contract::new(senna());
    //     contract.mint_nft(bob(),nft("first".to_string()));
    //     contract.mint_nft(bob(),nft("second".to_string()));
    //     testing_env!(get_context(bob(),env::storage_usage(),0,CREATE_AUCTION_FEE));
    //     contract.create_auction(2,60);
    //     contract.start_auction(1);
    //     assert_eq!(contract.get_auction_by_id(1).is_enabled,true,"");
    //     testing_env!(get_context(alice(),env::storage_usage(),5*s2ns,5_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(carol(),env::storage_usage(),10*s2ns,8_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(john(),env::storage_usage(),20*s2ns,5_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(smith(),env::storage_usage(),25*s2ns,6_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(james(),env::storage_usage(),26*s2ns,9_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(lili(),env::storage_usage(),27*s2ns,7_000_000_000_000_000_000_000_000));
    //     contract.commit_auction(1);
    //     testing_env!(get_context(senna(),env::storage_usage(),30*s2ns,0));
    //     contract.check_auctions();
    //     assert_eq!(contract.get_auction_by_id(1).winner,String::new(),"");
    //     testing_env!(get_context(senna(),env::storage_usage(),60*s2ns,0));
    //     contract.check_auctions();
    //     assert_eq!(contract.auction_by_id.get(&1).unwrap().close_price,6_000_000_000_000_000_000_000_000,"");
    //     assert_eq!(contract.get_auction_by_id(1).winner,smith(),"");
    //     assert_eq!(contract.get_token_by_id(2).owner_id,smith(),"");
    // }
    // #[test]
    // fn cal_test(){
    //     let  mut price:u32 = 0;
    //     let mut prices:Vec<u32> = vec![5,8,5,6,9,7,10,5,9,6,12,13,9,11];
    //     prices.sort();
    //     if prices.len()==1
    //         {
    //             price = prices[0].clone();
    //         }
    //     else if prices[0]!=prices[1] 
    //         {
    //             price = prices[0].clone();
    //         }
    //     else
    //         {
    //             for i in 1..(prices.len() -1)  
    //                 {
    //                     if price == 0 {
    //                     if i< (prices.len() -1)
    //                         {
    //                             if prices[i] != prices[i-1] && prices[i] != prices[i+1] 
    //                             {
    //                                 price = prices[i].clone();
    //                             }
    //                         }
    //                     else if i== prices.len()
    //                         {
    //                             if prices[i] != prices[i-1] 
    //                                 {
    //                                     price=prices[i].clone();
    //                                 }
    //                         }
    //                     else 
    //                         {
    //                             price = 0;
    //                         }
    //                     }
    //                 }
    //         }
    //     assert_eq!(price,7,"");
    // }
    
    
}
