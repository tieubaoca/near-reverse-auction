# near-reverse-auction
A contract allow user to mint NFT and hold an auction, the winner is the one with the single lowest bid.
</br>
<h3>Using this contract:</h3>
</br>
 <h4>Login account:</h4>
 </br>
  <code>near login</code>
  </br>
 You should use some accounts to try the auction.
 </br>
 <h4>Set variable:</h4>
 </br>
  <code>export ID=youracccount</code>
  </br>
 <h4>Deploy contract:</h4>
 </br>
  <code>near deploy --wasmFile res/contract.wasm --accountId $ID</code>
  </br>
 <h4>Init contract:</h4>
 </br>
  <code>near call $ID new '{"owner_id":"'$ID'"}' --accountId $ID</code>
  </br>
 <h4>You can use another account to mint nft, mint fee is 1N:</h4>
 </br>
  <code>near call $ID mint_nft '{"_owner_id":[account1],"_token_data":{"title":"title","description":"description"}}' --accountId [account1] --deposit 1</code>
  </br>
 <h4>Get list nft you own:</h4>
 </br>
  <code>near call $ID get_token_by_owner '{"_owner_id":[account1]}' --accountId [account1]</code>
  </br>
 <h4>Create an auction, create auction fee is 5N:</h4>
 </br>
  <code>near call $ID create_auction '{"_token_id":1,"_auction_during_seconds":60}' --accountId [account1] --deposit 5</code>
  </br>
 <h4>Get list auctions you own:</h4>
 </br>
  <code>near call $ID get_auction_by_owner '{"_auction_id":1}' --accountId [account1]</code>
  </br>
 <h4>Start an auction: </h4>
 </br>
  <code>near call $ID start_auction '{"_auction_id":1}' --accountId [account1]</code>
  </br>
 <h4>The participants now can commit their N:</h4>
 </br>
  <code>near call $ID commit_auction '{"_auction_id":1}' --accountId [account2] --deposit 1
 </br>
  near call $ID commit_auction '{"_auction_id":1}' --accountId [account2] --deposit 1.5</code>
  </br>
 <h4>The owner of the smart contract will continuously run the check function to check the status of the auctions:</h4>
 </br>
  <code>near call $ID check_auctions --accountId $ID</code>
