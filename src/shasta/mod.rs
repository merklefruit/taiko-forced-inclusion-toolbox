mod chainio;

use std::time::Duration;

use alloy::{
    consensus::{constants::GWEI_TO_WEI, Transaction},
    network::TransactionBuilder,
    primitives::{aliases::U24, Address, U256},
    providers::{Provider, ProviderBuilder, WalletProvider},
    rpc::types::TransactionRequest,
};
use futures::StreamExt;
use taiko_protocol::shasta::manifest::{BlockManifest, DerivationSourceManifest};
use tokio::time::sleep;

use crate::{
    blob::create_blob_sidecar_from_data_async,
    cli::{
        Cmd::{MonitorQueue, ReadQueue, Send, Spam},
        SendCmdOptions, SpamCmdOptions,
    },
    wallet_provider::DefaultWalletProvider,
};

use chainio::IForcedInclusionStore::{self, ForcedInclusionSaved, IForcedInclusionStoreInstance};
use chainio::LibBlobs::BlobReference;

pub async fn handle_command(cli: crate::cli::Cli) -> eyre::Result<()> {
    let l1 = ProviderBuilder::new()
        .wallet(cli.l1_private_key)
        .connect_http(cli.l1_rpc_url);
    let l2 = ProviderBuilder::new()
        .wallet(cli.l2_private_key)
        .connect_http(cli.l2_rpc_url);

    let store = IForcedInclusionStore::new(cli.forced_inclusion_store_address, l1);

    match cli.command {
        // shasta commands
        ReadQueue => read_queue(&store).await,
        MonitorQueue => monitor_queue(&store).await,
        Send(opts) => send_one(opts, &l2, &store).await,
        Spam(opts) => spam(opts, &l2, &store).await,
    }
}

/// Send a forced inclusion transaction.
pub async fn send_one(
    opts: SendCmdOptions,
    l2: &DefaultWalletProvider,
    store: &IForcedInclusionStoreInstance<DefaultWalletProvider>,
) -> eyre::Result<()> {
    // Generate the L2 transaction to be force-included. Make it a simple transfer of 1 gwei.
    let mut l2_tx_req = TransactionRequest::default()
        .to(Address::ZERO)
        .value(U256::from(GWEI_TO_WEI));

    // If a nonce delta is provided, calculate the nonce manually instead of using the
    // default `CachedNonceManager` value.
    if opts.nonce_delta > 0 {
        let sender = l2.wallet().default_signer().address();
        let pending_nonce = l2.get_transaction_count(sender).pending().await?;
        l2_tx_req.set_nonce(pending_nonce + opts.nonce_delta);
    }

    let l2_tx = l2.fill(l2_tx_req).await?.try_into_envelope()?;
    println!(
        "🔍 L2 tx to be force-included: nonce={}, hash={}",
        l2_tx.nonce(),
        l2_tx.hash()
    );

    // Build the proposal manifest.
    let block_manifests = vec![BlockManifest {
        timestamp: 0,
        coinbase: Address::ZERO,
        anchor_block_number: 0,
        gas_limit: 0,
        transactions: vec![l2_tx],
    }];

    let manifest = DerivationSourceManifest {
        blocks: block_manifests,
    };

    let manifest_data = manifest.encode_and_compress()?;

    // Prepare the sidecar for the forced inclusion
    let sidecar = create_blob_sidecar_from_data_async(manifest_data.into()).await?;

    // Get the required fee for the forced inclusion
    let fee_wei = U256::from(store.getCurrentForcedInclusionFee().call().await? * GWEI_TO_WEI);

    let blob_ref = BlobReference {
        blobStartIndex: 0,
        numBlobs: sidecar.blobs.len() as u16,
        offset: U24::ZERO,
    };

    // Send the forced inclusion transaction on L1
    match store
        .saveForcedInclusion(blob_ref)
        .sidecar(sidecar)
        .value(fee_wei)
        .send()
        .await
    {
        Ok(tx) => {
            let receipt = tx.get_receipt().await?;
            if receipt.status() {
                println!(
                    "✅ Forced inclusion batch sent successfully! Hash: {}",
                    receipt.transaction_hash
                );
            } else {
                println!(
                    "❌ Forced inclusion batch failed! Status: {}",
                    receipt.transaction_hash
                );
            }
        }
        Err(e) => {
            println!("❌ Forced inclusion batch failed! Error: {e}",);
        }
    }
    Ok(())
}

/// Read the forced inclusion queue from the contract.
pub async fn read_queue(store: &IForcedInclusionStoreInstance<DefaultWalletProvider>) -> eyre::Result<()> {
    let state = store.getForcedInclusionState().call().await?;
    let head = state.head_.to::<u64>();
    let size = state.tail_.saturating_sub(state.head_);

    if size == 0 {
        println!("Forced inclusion queue is empty");
        return Ok(());
    }

    let forced_inclusions = store.getForcedInclusions(state.head_, size).call().await?;
    for (i, fi) in forced_inclusions.iter().enumerate() {
        println!("Forced inclusion {}: {:?}\n", head + i as u64, fi);
    }

    Ok(())
}

/// Monitor events in the forced inclusion queue
pub async fn monitor_queue(store: &IForcedInclusionStoreInstance<DefaultWalletProvider>) -> eyre::Result<()> {
    let saved = store.ForcedInclusionSaved_filter().filter;

    let mut saved_sub = store.provider().watch_logs(&saved).await?.into_stream();

    println!("Monitoring forced inclusion queue...");
    loop {
        tokio::select! {
            Some(events) = saved_sub.next() => {
                if let Some(event) = events.first() {
                    let decoded = event.log_decode::<ForcedInclusionSaved>()?;
                    println!("New forced inclusion saved: {:?}", decoded.data().forcedInclusion);
                }
            }
        }
    }
}

/// Send forced inclusion transactions in a loop.
pub async fn spam(opts: SpamCmdOptions,
    l2: &DefaultWalletProvider,
    store: &IForcedInclusionStoreInstance<DefaultWalletProvider>,
) -> eyre::Result<()> {
    let send_opts = SendCmdOptions::default();

    loop {
        // NOTE: by using the default `CachedNonceManager`, the nonce will be incremented
        // automatically by the provider without making new RPC calls.
        if let Err(e) = send_one(send_opts, l2, store).await {
            eprintln!("Error sending forced-inclusion: {e:?}");
            return Err(e);
        }

        sleep(Duration::from_secs(opts.interval_secs)).await;
    }
}
