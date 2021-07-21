# near-reverse-auction
A contract allow user to mint NFT and hold an auction, the winner is the one with the single lowest bid.
</br>
Using this contract:
 Login account:
  near login
 You should use some accounts to try the auction.
 Set variable: 
  export ID=youracccount
 Deploy contract:
  near deploy --wasmFile res/contract.wasm --accountId $ID
 Init contract:
  near call $ID new '{"owner_id":"'$ID'"}' --accountId $ID
 You can use another account to mint nft, mint fee is 1N:
  near call $ID mint_nft '{"_owner_id":[account1],"_token_data":{"title":"title","description":"description"}}' --accountId [account1] --deposit 1
 Get list nft you own:
  near call $ID get_token_by_owner '{"_owner_id":[account1]}' --accountId [account1]
 Create an auction, create auction fee is 5N:
  near call $ID create_auction '{"_token_id":1,"_auction_during_seconds":60}' --accountId [account1] --deposit 5
 Get list auctions you own:
  near call $ID get_auction_by_owner '{"_auction_id":1}' --accountId [account1]
 Start an auction: 
  near call $ID start_auction '{"_auction_id":1}' --accountId [account1]
 The participants now can commit their N:
  near call $ID commit_auction '{"_auction_id":1}' --accountId [account2] --deposit 1
  near call $ID commit_auction '{"_auction_id":1}' --accountId [account2] --deposit 1.5
 The owner of the smart contract will continuously run the check function to check the status of the auctions:
  near call $ID check_auctions --accountId $ID
