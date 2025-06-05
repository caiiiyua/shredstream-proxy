use jito_protos::shredstream::{
    shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use log::info;

pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::builder().init();
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
                    info!("Deserialization failed with err: {e}");
                    continue;
                }
            };
        let entries_size = entries.len();
        let transactions = entries.iter().map(|e| e.transactions.len()).sum::<usize>();
        for entry in entries {
            for transaction in entry.transactions {
                if transaction.message.static_account_keys().contains(&PUMPFUN_MINT_AUTHORITY) {
                    let signature = transaction.signatures.first().unwrap();
                    info!("[{}, {}, {}][{:?}]", slot_entry.slot, entries_size, transactions, signature);
                }
            }
        }
    }
    Ok(())
}
