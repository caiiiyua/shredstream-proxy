use std::collections::HashSet;
use std::sync::Arc;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use jito_protos::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
};

pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut client = ShredstreamProxyClient::connect("http://127.0.0.1:9999")
        .await
        .unwrap();
    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await
        .unwrap()
        .into_inner();

    while let Some(slot_entry) = stream.message().await.unwrap() {
        let entries =
            match bincode::deserialize::<Vec<solana_entry::entry::Entry>>(&slot_entry.entries) {
                Ok(e) => e,
                Err(e) => {
                    println!("Deserialization failed with err: {e}");
                    continue;
                }
            };
        println!(
            "slot {}, entries: {}, transactions: {}",
            slot_entry.slot,
            entries.len(),
            entries.iter().map(|e| e.transactions.len()).sum::<usize>()
        );
        tokio::spawn(async move {
            for entry in entries {
                for transaction in entry.transactions {
                    if transaction.message.static_account_keys().contains(&PUMPFUN_MINT_AUTHORITY) {
                        let signature = transaction.signatures.first().unwrap();
                        println!("[{}][{:?}]", slot_entry.slot, signature);
                    }
                }
            }
        });
    }
    Ok(())
}
