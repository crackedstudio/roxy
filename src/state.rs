use linera_sdk::linera_base_types::{AccountOwner, Amount, ChainId, Timestamp};
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type MarketId = u64;
pub type PlayerId = AccountOwner;
pub type OutcomeId = u32;
pub type GuildId = u64;
pub type AchievementId = u32;

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::InputObject)]
pub struct GameConfig {
    pub admin: Option<AccountOwner>,
    pub initial_player_tokens: Amount,
    pub daily_login_reward: Amount,
    pub market_creation_cost: Amount,
    pub min_market_duration_seconds: u64,
    pub max_outcomes_per_market: usize,
    pub oracle_voting_duration_seconds: u64,
    pub min_oracle_voters: u32,
    pub market_creator_fee_percent: u8,
    pub platform_fee_percent: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            admin: None,
            initial_player_tokens: Amount::from_tokens(100),
            daily_login_reward: Amount::from_tokens(10),
            market_creation_cost: Amount::from_tokens(100),
            min_market_duration_seconds: 300,
            max_outcomes_per_market: 10,
            oracle_voting_duration_seconds: 3600,
            min_oracle_voters: 3,
            market_creator_fee_percent: 2,
            platform_fee_percent: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct Market {
    pub id: MarketId,
    pub creator: PlayerId,
    pub title: String,
    pub amount: Amount,  // Amount of points to sell or buy
    pub fee_percent: u8, // Fee percentage that seller wants to charge (0-100)
    pub creation_time: Timestamp,
    pub status: MarketStatus,
    pub total_liquidity: Amount, // Total points available in this market
    pub positions: BTreeMap<PlayerId, PlayerPosition>,
    pub total_participants: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketType {
    QuickPrediction,
    TournamentMarket,
    SeasonalEvent,
    PvPChallenge {
        challenger: PlayerId,
        challenged: PlayerId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, async_graphql::Enum)]
pub enum MarketStatus {
    Active,
    Closed,
    Resolved,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, async_graphql::Enum)]
pub enum ResolutionMethod {
    OracleVoting,
    Automated,
    CreatorDecides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub id: OutcomeId,
    pub name: String,
    pub total_shares: Amount,
    pub current_price: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPosition {
    pub shares_by_outcome: BTreeMap<OutcomeId, Amount>,
    pub total_invested: Amount,
    pub entry_time: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct Player {
    pub id: PlayerId,
    pub display_name: Option<String>,
    pub registration_time: Timestamp,
    pub last_login: Timestamp,
    pub token_balance: Amount,
    pub total_earned: Amount,
    pub total_spent: Amount,
    pub level: u32,
    pub experience_points: u64,
    pub reputation: u64,
    pub markets_participated: u64,
    pub markets_won: u64,
    pub total_profit: Amount,
    pub win_streak: u32,
    pub best_win_streak: u32,
    pub guild_id: Option<GuildId>,
    pub achievements_earned: Vec<AchievementId>,
    pub active_markets: Vec<MarketId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct Leaderboard {
    pub top_traders: Vec<LeaderboardEntry>,
    pub top_guilds: Vec<GuildLeaderboardEntry>,
    pub last_updated: Timestamp,
}

impl Default for Leaderboard {
    fn default() -> Self {
        Self {
            top_traders: Vec::new(),
            top_guilds: Vec::new(),
            last_updated: Timestamp::from(0u64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct LeaderboardEntry {
    pub player_id: PlayerId,
    pub display_name: Option<String>,
    pub total_profit: Amount,
    pub win_rate: f64,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct Guild {
    pub id: GuildId,
    pub name: String,
    pub founder: PlayerId,
    pub members: Vec<PlayerId>,
    pub creation_time: Timestamp,
    pub total_guild_profit: Amount,
    pub guild_level: u32,
    pub shared_pool: Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct GuildLeaderboardEntry {
    pub guild_id: GuildId,
    pub name: String,
    pub total_profit: Amount,
    pub member_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleVoting {
    pub market_id: MarketId,
    pub voting_start: Timestamp,
    pub voting_end: Timestamp,
    pub votes: BTreeMap<OutcomeId, WeightedVotes>,
    pub voters: Vec<PlayerId>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedVotes {
    pub total_weight: u64,
    pub voter_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: AchievementId,
    pub name: String,
    pub description: String,
    pub reward_tokens: Amount,
    pub reward_xp: u64,
    pub requirement: AchievementRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementRequirement {
    CreateMarket,    // Create first market
    FirstBuy,        // Make first buy
    FirstSell,       // Make first sell
    JoinGuild,       // Join a guild
    ReachLevel(u32), // Reach specific level (e.g., level 2, 3, 4...)
    // Legacy requirements (kept for backward compatibility)
    WinMarkets(u64),
    WinStreak(u32),
    TotalProfit(Amount),
    ParticipateInMarkets(u64),
    CreateMarkets(u64),
}

// ============================================================================
// Price Prediction Types
// ============================================================================

/// Represents the predicted outcome of market price movement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, async_graphql::Enum)]
pub enum PriceOutcome {
    Rise,    // Price increased
    Fall,    // Price decreased
    Neutral, // Price stayed the same
}

/// Represents the period type for predictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum PredictionPeriod {
    Daily,   // Daily prediction (24 hours)
    Weekly,  // Weekly prediction (7 days)
    Monthly, // Monthly prediction (30 days)
}

/// Stores a player's prediction for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPrediction {
    pub player_id: PlayerId,
    pub period: PredictionPeriod,
    pub outcome: PriceOutcome,
    pub prediction_time: Timestamp,
    pub period_start: Timestamp, // Start timestamp of the prediction period
    pub resolved: bool,          // Whether the prediction has been resolved
    pub correct: Option<bool>,   // None if not resolved, Some(true/false) if resolved
}

/// Stores market price data at a specific timestamp
/// Used by oracle/admin to submit actual market prices for verification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarketPrice {
    pub price: Amount, // Price as an Amount (using token units as price units)
    pub timestamp: Timestamp,
}

/// Stores price data for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodPriceData {
    pub period_start: Timestamp,
    pub period_end: Timestamp,
    pub start_price: Option<MarketPrice>, // Price at period start
    pub end_price: Option<MarketPrice>,   // Price at period end
    pub outcome: Option<PriceOutcome>,    // Calculated outcome based on price change
    pub resolved: bool,
}

/// Key for storing predictions: (player_id, period_type, period_start_timestamp)
pub type PredictionKey = (PlayerId, PredictionPeriod, Timestamp);

/// Key for storing period price data: (period_type, period_start_timestamp)
pub type PeriodKey = (PredictionPeriod, Timestamp);

#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct PredictionMarketState {
    pub config: RegisterView<GameConfig>,
    pub markets: MapView<MarketId, Market>,
    pub players: MapView<PlayerId, Player>,
    pub leaderboard: RegisterView<Leaderboard>,
    pub guilds: MapView<GuildId, Guild>,
    pub oracle_votes: MapView<MarketId, OracleVoting>,
    pub achievements: MapView<AchievementId, Achievement>,
    pub total_supply: RegisterView<Amount>,
    pub next_market_id: RegisterView<MarketId>,
    // Price prediction state
    pub predictions: MapView<String, PlayerPrediction>, // Key: format!("{player_id}_{period}_{period_start}")
    pub period_prices: MapView<String, PeriodPriceData>, // Key: format!("{period}_{period_start}")
    pub current_market_price: RegisterView<MarketPrice>, // Current market price (updated by oracle)
    // Global state for horizontal scaling (cross-chain)
    pub global_players: MapView<PlayerId, GlobalPlayerInfo>, // Registry of all players across all chains
    pub global_markets: MapView<MarketId, GlobalMarketInfo>, // Registry of all markets across all chains
    pub global_guilds: MapView<GuildId, GlobalGuildInfo>, // Registry of all guilds across all chains
    pub global_leaderboard: RegisterView<Leaderboard>, // Aggregated leaderboard across all chains
    // Chain registry for cross-chain messaging
    pub subscribed_chains: MapView<ChainId, Timestamp>, // Registry of chains that have the application
}

/// Global player information for cross-chain coordination
#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct GlobalPlayerInfo {
    pub player_id: PlayerId,
    pub display_name: Option<String>,
    pub chain_id: ChainId,
    pub total_earned: Amount,
    pub total_profit: Amount,
    pub level: u32,
    pub last_updated: Timestamp,
}

/// Global market information for cross-chain coordination
#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct GlobalMarketInfo {
    pub market_id: MarketId,
    pub creator: PlayerId,
    pub title: String,
    pub chain_id: ChainId,
    pub status: MarketStatus,
    pub created_at: Timestamp,
}

/// Global guild information for cross-chain coordination
#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct GlobalGuildInfo {
    pub guild_id: GuildId,
    pub name: String,
    pub founder: PlayerId,
    pub chain_id: ChainId,
    pub member_count: u32,
    pub total_guild_profit: Amount,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Local events (same chain)
    MarketCreated {
        market_id: MarketId,
        creator: PlayerId,
    },
    MarketResolved {
        market_id: MarketId,
        winning_outcome: OutcomeId,
    },
    TradeExecuted {
        player_id: PlayerId,
        market_id: MarketId,
        outcome_id: OutcomeId,
        shares: Amount,
        price: Amount,
    },
    PlayerLeveledUp {
        player_id: PlayerId,
        new_level: u32,
    },
    AchievementUnlocked {
        player_id: PlayerId,
        achievement_id: AchievementId,
    },
    GuildCreated {
        guild_id: GuildId,
        name: String,
    },
    PredictionMade {
        player_id: PlayerId,
        period: PredictionPeriod,
        outcome: PriceOutcome,
    },
    PredictionResolved {
        player_id: PlayerId,
        period: PredictionPeriod,
        correct: bool,
    },
    // Cross-chain messages for horizontal scaling
    GlobalMarketCreated {
        market_id: MarketId,
        creator: PlayerId,
        title: String,
        chain_id: ChainId,
    },
    GlobalPlayerRegistered {
        player_id: PlayerId,
        display_name: Option<String>,
        chain_id: ChainId,
    },
    GlobalPlayerUpdated {
        player_id: PlayerId,
        total_earned: Amount,
        total_profit: Amount,
        level: u32,
        chain_id: ChainId,
    },
    GlobalGuildCreated {
        guild_id: GuildId,
        name: String,
        founder: PlayerId,
        chain_id: ChainId,
    },
    GlobalLeaderboardUpdate {
        player_id: PlayerId,
        total_profit: Amount,
        win_rate: f64,
        level: u32,
        chain_id: ChainId,
    },
    GlobalPriceUpdate {
        price: Amount,
        timestamp: Timestamp,
        chain_id: ChainId,
    },
    // Chain registration for cross-chain coordination
    ChainRegistered {
        chain_id: ChainId,
        timestamp: Timestamp,
    },
}
