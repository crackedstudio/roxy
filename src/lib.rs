use async_graphql::{Request, Response};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{Amount, ContractAbi, ServiceAbi},
};
use serde::{Deserialize, Serialize};

pub mod state;

// Re-export types for convenience
pub use state::{
    AchievementId, GameConfig, GlobalGuildInfo, GlobalMarketInfo, GlobalPlayerInfo, Guild, GuildId,
    Leaderboard, Market, MarketId, MarketStatus, MarketType, OutcomeId, Player, PlayerId,
    PlayerPrediction, PredictionPeriod, PriceOutcome, ResolutionMethod,
};

pub struct PredictiveManagerAbi;

impl ContractAbi for PredictiveManagerAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for PredictiveManagerAbi {
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    // Player operations
    RegisterPlayer {
        display_name: Option<String>,
    },
    UpdateProfile {
        display_name: Option<String>,
    },
    ClaimDailyReward,

    // Market operations
    CreateMarket {
        title: String,
        amount: Amount,
        fee_percent: u8, // Fee percentage seller wants to charge (0-100)
    },
    BuyShares {
        market_id: MarketId, // Market to buy points from
        amount: Amount,
    },
    SellShares {
        market_id: MarketId, // Market to sell points to
        amount: Amount,
    },

    // Point minting (Admin only)
    MintPoints {
        amount: Amount,
    },

    // Guild operations
    CreateGuild {
        name: String,
    },
    JoinGuild {
        guild_id: GuildId,
    },
    LeaveGuild,
    ContributeToGuild {
        amount: Amount,
    },

    // Admin operations
    UpdateGameConfig {
        config: GameConfig,
    },

    // Price prediction operations
    PredictDailyOutcome {
        outcome: PriceOutcome,
    },
    PredictWeeklyOutcome {
        outcome: PriceOutcome,
    },
    PredictMonthlyOutcome {
        outcome: PriceOutcome,
    },

    // Oracle/Admin operations for price updates
    UpdateMarketPrice {
        price: Amount,
    },
}
