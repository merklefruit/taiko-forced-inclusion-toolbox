use alloy::{
    network::EthereumWallet,
    providers::{
        RootProvider,
        fillers::{FillProvider, JoinFill, WalletFiller},
        utils::JoinedRecommendedFillers,
    },
};

/// Alias to the joined recommended fillers + wallet filler for Ethereum wallets.
pub type JoinedWalletFillers = JoinFill<JoinedRecommendedFillers, WalletFiller<EthereumWallet>>;

/// Alias to the default wallet provider with all recommended fillers (read + write).
pub type DefaultWalletProvider = FillProvider<JoinedWalletFillers, RootProvider>;
