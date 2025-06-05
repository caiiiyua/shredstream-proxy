use std::fmt;
use jito_protos::shredstream::{shredstream_proxy_client::ShredstreamProxyClient, PumpfunTx, SubscribeEntriesRequest, SubscribePumpfunTxRequest};
use solana_sdk::{bs58, pubkey};
use solana_sdk::pubkey::Pubkey;
use log::info;

pub const PUMPFUN_MINT_AUTHORITY: Pubkey = pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKt1eokM");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::builder()
        .format_timestamp_millis()
        .init();
    let mut client = ShredstreamProxyClient::connect("http://127.0.0.1:9999")
        .await
        .unwrap();

    let mut client_a = client.clone();
    tokio::spawn(async move {
        let mut stream = client_a
            .subscribe_pumpfun_tx(SubscribePumpfunTxRequest {})
            .await
            .unwrap()
            .into_inner();

        while let Some(pumpfun_tx) = stream.message().await.unwrap() {
            let pumpfun_tx: PumpfunTransaction = pumpfun_tx.into();
            info!("pumpfun_tx: {}", pumpfun_tx);
        }
    });

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

impl From<PumpfunTransaction> for PumpfunTx {
    fn from(tx: PumpfunTransaction) -> Self {
        PumpfunTx {
            token_amount: tx.token_amount,
            sol_paid: tx.sol_paid,
            slot: tx.slot,
            fee_account: tx.fee_account.to_vec(),
            token_mint: tx.token_mint.to_vec(),
            bonding_curve: tx.bonding_curve.to_vec(),
            bonding_curve_vault: tx.bonding_curve_vault.to_vec(),
            dev_account: tx.dev_account.to_vec(),
            creator_vault: tx.creator_vault.to_vec(),
        }
    }
}

#[derive(Debug, Clone)]
struct PumpfunTransaction {
    token_amount: u64,
    sol_paid: u64,
    slot: u64,
    fee_account: [u8; 32],
    token_mint: [u8; 32],
    dev_account: [u8; 32],
    bonding_curve: [u8; 32],
    bonding_curve_vault: [u8; 32],
    creator_vault: [u8; 32],
    block_hash: [u8; 32],
}

impl From<PumpfunTx> for PumpfunTransaction {
    fn from(tx: PumpfunTx) -> Self {
        PumpfunTransaction {
            token_amount: tx.token_amount,
            sol_paid: tx.sol_paid,
            slot: tx.slot, // Slot is not present in PumpfunTx, set to 0 or handle accordingly
            fee_account: tx.fee_account.try_into().expect("Invalid fee account length"),
            token_mint: tx.token_mint.try_into().expect("Invalid token mint length"),
            bonding_curve: tx.bonding_curve.try_into().expect("Invalid bonding curve length"),
            bonding_curve_vault: tx.bonding_curve_vault.try_into().expect("Invalid bonding curve vault length"),
            dev_account: tx.dev_account.try_into().expect("Invalid dev account length"),
            creator_vault: tx.creator_vault.try_into().expect("Invalid creator vault length"),
            block_hash: [0; 32], // Block hash is not present in PumpfunTx, set to zero or handle accordingly
        }
    }
}

impl fmt::Display for PumpfunTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PumpfunTransaction {{\n\
             \ttoken_amount: {},\n\
             \tsol_paid: {},\n\
             \tfee_account: {:?},\n\
             \ttoken_mint: {:?},\n\
             \tbonding_curve: {:?},\n\
             \tbonding_curve_vault: {:?},\n\
             \tdev_account: {:?},\n\
             \tcreator_vault: {:?},\n\
             \tblock_hash: {:?}\n\
             }}",
            self.token_amount,
            self.sol_paid,
            bs58::encode(self.fee_account).into_string(),
            bs58::encode(self.token_mint).into_string(),
            bs58::encode(self.bonding_curve).into_string(),
            bs58::encode(self.bonding_curve_vault).into_string(),
            bs58::encode(self.dev_account).into_string(),
            bs58::encode(self.creator_vault).into_string(),
            bs58::encode(self.block_hash).into_string(),
        )
    }
}
