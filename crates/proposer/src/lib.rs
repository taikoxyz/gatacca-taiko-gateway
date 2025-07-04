//! Handles creating and landing L1 blockPropose transactions.

use alloy_provider::ProviderBuilder;
use alloy_signer_local::PrivateKeySigner;
use client::L1Client;
use manager::ProposerManager;
use pc_common::{
    beacon::BeaconHandle,
    config::{ProposerConfig, StaticConfig, TaikoConfig},
    proposer::ProposalRequest,
    runtime::spawn,
};
use tokio::sync::mpsc::UnboundedReceiver;

mod client;
mod error;
mod manager;

pub async fn start_proposer(
    config: &StaticConfig,
    taiko_config: TaikoConfig,
    proposer_signer: PrivateKeySigner,
    new_blocks_rx: UnboundedReceiver<ProposalRequest>,
    beacon_handle: BeaconHandle,
) -> eyre::Result<()> {
    let proposer_config: ProposerConfig = config.into();

    let includer = L1Client::new(
        config.l1.rpc_url.clone(),
        taiko_config.l1_contract,
        proposer_signer,
        proposer_config.l1_safe_lag,
        config.l2.router_contract,
    )
    .await?;

    let l2_provider = ProviderBuilder::new()
        .disable_recommended_fillers()
        .on_http(taiko_config.rpc_url[0].clone());

    let proposer = ProposerManager::new(
        proposer_config,
        includer,
        l2_provider,
        taiko_config,
        config.gateway.l1_safe_lag,
        beacon_handle,
    );

    // start proposer
    spawn(proposer.run(new_blocks_rx));

    Ok(())
}
