#![cfg_attr(target_arch = "wasm32", no_main)]

use linera_sdk::views::ViewError;
use linera_sdk::{
    linera_base_types::{Amount, Timestamp, WithContractAbi},
    views::View,
    Contract, ContractRuntime,
};
use predictive_manager::state::*;
use std::collections::BTreeMap;
use thiserror::Error;

// ============================================================================
// Errors
// ============================================================================

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("player already exists")]
    PlayerAlreadyExists,
    #[error("daily reward already claimed")]
    DailyRewardAlreadyClaimed,
    #[error("invalid outcome count")]
    InvalidOutcomeCount,
    #[error("duration too short")]
    DurationTooShort,
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("market not active")]
    MarketNotActive,
    #[error("market ended")]
    MarketEnded,
    #[error("invalid outcome")]
    InvalidOutcome,
    #[error("slippage exceeded")]
    SlippageExceeded,
    #[error("no position")]
    NoPosition,
    #[error("insufficient shares")]
    InsufficientShares,
    #[error("market not ready for voting")]
    MarketNotReadyForVoting,
    #[error("invalid resolution method")]
    InvalidResolutionMethod,
    #[error("already voted")]
    AlreadyVoted,
    #[error("market not ended")]
    MarketNotEnded,
    #[error("player not found")]
    PlayerNotFound,
    #[error("market not found")]
    MarketNotFound,
    #[error("guild not found")]
    GuildNotFound,
    #[error("already in guild")]
    AlreadyInGuild,
    #[error("not a guild member")]
    NotGuildMember,
    #[error("not admin")]
    NotAdmin,
    #[error("oracle not ready")]
    OracleNotReady,
    #[error("not resolved")]
    NotResolved,
    #[error("no winnings")]
    NoWinnings,
    #[error("insufficient level")]
    InsufficientLevel,
    #[error(transparent)]
    View(#[from] ViewError),
}

// ============================================================================
// Contract Implementation
// ============================================================================

pub struct PredictionMarketContract {
    state: PredictionMarketState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(PredictionMarketContract);

impl WithContractAbi for PredictionMarketContract {
    type Abi = predictive_manager::PredictiveManagerAbi;
}

impl Contract for PredictionMarketContract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = GameConfig;
    type EventValue = ();

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = PredictionMarketState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        PredictionMarketContract { state, runtime }
    }

    async fn instantiate(&mut self, config: GameConfig) {
        // If admin is not set, try to set it to the authenticated signer (deployer)
        // In production, admin should be set explicitly in GameConfig when creating the application
        let mut final_config = config;
        if final_config.admin.is_none() {
            if let Some(deployer) = self.runtime.authenticated_signer() {
                final_config.admin = Some(deployer);
            }
        }

        self.state.config.set(final_config);
        self.state.total_supply.set(Amount::ZERO);
        self.state.next_market_id.set(0);
        let _ = self.initialize_achievements().await;
        self.state.leaderboard.set(Leaderboard {
            top_traders: Vec::new(),
            top_guilds: Vec::new(),
            last_updated: self.runtime.system_time(),
        });

        // Initialize enhanced leaderboard
        self.update_enhanced_leaderboard().await;
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        let player_id = self.runtime.authenticated_signer().unwrap();
        let current_time = self.runtime.system_time();

        // Register this chain if not already registered (for cross-chain coordination)
        self.ensure_chain_registered().await;

        match operation {
            predictive_manager::Operation::RegisterPlayer { display_name } => {
                let _ = self
                    .register_player(player_id, display_name, current_time)
                    .await;
            }
            predictive_manager::Operation::UpdateProfile { display_name } => {
                let _ = self.update_player_profile(player_id, display_name).await;
            }
            predictive_manager::Operation::ClaimDailyReward => {
                let _ = self.claim_daily_reward(player_id, current_time).await;
            }
            predictive_manager::Operation::CreateMarket {
                title,
                amount,
                fee_percent,
            } => {
                let _ = self
                    .create_market(player_id, title, amount, fee_percent, current_time)
                    .await;
            }
            predictive_manager::Operation::BuyShares { market_id, amount } => {
                let _ = self
                    .buy_shares(player_id, market_id, amount, current_time)
                    .await;
            }
            predictive_manager::Operation::SellShares { market_id, amount } => {
                let _ = self
                    .sell_shares(player_id, market_id, amount, current_time)
                    .await;
            }
            predictive_manager::Operation::MintPoints { amount } => {
                let _ = self.mint_points(player_id, amount).await;
            }
            predictive_manager::Operation::CreateGuild { name } => {
                let _ = self.create_guild(player_id, name, current_time).await;
            }
            predictive_manager::Operation::JoinGuild { guild_id } => {
                let _ = self.join_guild(player_id, guild_id).await;
            }
            predictive_manager::Operation::LeaveGuild => {
                let _ = self.leave_guild(player_id).await;
            }
            predictive_manager::Operation::ContributeToGuild { amount } => {
                let _ = self.contribute_to_guild(player_id, amount).await;
            }
            predictive_manager::Operation::UpdateGameConfig { config } => {
                let _ = self.update_game_config(player_id, config).await;
            }
            predictive_manager::Operation::PredictDailyOutcome { outcome } => {
                let _ = self
                    .predict_daily_outcome(player_id, outcome, current_time)
                    .await;
            }
            predictive_manager::Operation::PredictWeeklyOutcome { outcome } => {
                let _ = self
                    .predict_weekly_outcome(player_id, outcome, current_time)
                    .await;
            }
            predictive_manager::Operation::PredictMonthlyOutcome { outcome } => {
                let _ = self
                    .predict_monthly_outcome(player_id, outcome, current_time)
                    .await;
            }
            predictive_manager::Operation::UpdateMarketPrice { price } => {
                let _ = self
                    .update_market_price(player_id, price, current_time)
                    .await;
            }
        }
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            // Local events (same chain)
            Message::MarketCreated { .. } => {}
            Message::MarketResolved { .. } => {}
            Message::TradeExecuted { .. } => {}
            Message::PlayerLeveledUp { .. } => {}
            Message::AchievementUnlocked { .. } => {}
            Message::GuildCreated { .. } => {}
            Message::PredictionMade { .. } => {}
            Message::PredictionResolved { .. } => {}
            // Cross-chain messages for horizontal scaling
            Message::GlobalMarketCreated {
                market_id,
                creator,
                title,
                chain_id,
                message_id,
            } => {
                // Idempotency check: Skip if message already processed
                if self.is_message_processed(&message_id).await {
                    return; // Message already processed, skip
                }

                // Idempotency check: Only create if market doesn't already exist
                if self.state.global_markets.get(&market_id).await.ok().flatten().is_none() {
                    // Update global market registry
                    let global_market = GlobalMarketInfo {
                        market_id,
                        creator,
                        title,
                        chain_id,
                        status: MarketStatus::Active,
                        created_at: self.runtime.system_time(),
                    };
                    let _ = self.state.global_markets.insert(&market_id, global_market);
                }

                // Mark message as processed
                let _ = self.mark_message_processed(&message_id).await;
            }
            Message::GlobalPlayerRegistered {
                player_id,
                display_name,
                chain_id,
                message_id,
            } => {
                // Idempotency check: Skip if message already processed
                if self.is_message_processed(&message_id).await {
                    return; // Message already processed, skip
                }

                // Idempotency check: Only register if player doesn't already exist
                if self.state.global_players.get(&player_id).await.ok().flatten().is_none() {
                    // Update global player registry
                    let global_player = GlobalPlayerInfo {
                        player_id,
                        display_name,
                        chain_id,
                        total_earned: Amount::ZERO,
                        total_profit: Amount::ZERO,
                        level: 1,
                        last_updated: self.runtime.system_time(),
                    };
                    let _ = self.state.global_players.insert(&player_id, global_player);
                }

                // Mark message as processed
                let _ = self.mark_message_processed(&message_id).await;
            }
            Message::GlobalPlayerUpdated {
                player_id,
                total_earned,
                total_profit,
                level,
                chain_id,
                timestamp,
                message_id,
            } => {
                // Idempotency check: Skip if message already processed
                if self.is_message_processed(&message_id).await {
                    return; // Message already processed, skip
                }

                // Update global player info with timestamp-based conflict resolution
                if let Some(mut global_player) = self.state.global_players.get(&player_id).await.ok().flatten() {
                    // Only update if incoming data is newer (timestamp-based conflict resolution)
                    if timestamp > global_player.last_updated {
                        global_player.total_earned = total_earned;
                        global_player.total_profit = total_profit;
                        global_player.level = level;
                        global_player.chain_id = chain_id;
                        global_player.last_updated = timestamp; // Use message timestamp, not local time
                        let _ = self.state.global_players.insert(&player_id, global_player);
                        // Update global leaderboard
                        self.update_global_leaderboard().await;
                    }
                    // If timestamp is older or equal, ignore the update (already have newer data)
                }

                // Mark message as processed
                let _ = self.mark_message_processed(&message_id).await;
            }
            Message::GlobalGuildCreated {
                guild_id,
                name,
                founder,
                chain_id,
                message_id,
            } => {
                // Idempotency check: Skip if message already processed
                if self.is_message_processed(&message_id).await {
                    return; // Message already processed, skip
                }

                // Idempotency check: Only create if guild doesn't already exist
                if self.state.global_guilds.get(&guild_id).await.ok().flatten().is_none() {
                    // Update global guild registry
                    let global_guild = GlobalGuildInfo {
                        guild_id,
                        name,
                        founder,
                        chain_id,
                        member_count: 1,
                        total_guild_profit: Amount::ZERO,
                        created_at: self.runtime.system_time(),
                    };
                    let _ = self.state.global_guilds.insert(&guild_id, global_guild);
                }

                // Mark message as processed
                let _ = self.mark_message_processed(&message_id).await;
            }
            Message::GlobalLeaderboardUpdate {
                player_id,
                total_profit,
                win_rate,
                level,
                chain_id: _,
            } => {
                // Update global leaderboard with cross-chain data
                self.update_global_leaderboard_with_player(player_id, total_profit, win_rate, level).await;
            }
            Message::GlobalPriceUpdate { price, timestamp, chain_id: _, message_id } => {
                // Idempotency check: Skip if message already processed
                if self.is_message_processed(&message_id).await {
                    return; // Message already processed, skip
                }

                // Price timestamp validation: Only update if incoming price is newer
                let current_price = self.state.current_market_price.get();
                if timestamp > current_price.timestamp {
                    // Update global price (propagate across chains)
                    let market_price = MarketPrice { price, timestamp };
                    self.state.current_market_price.set(market_price);
                }
                // If timestamp is older or equal, ignore the update (already have newer price)

                // Mark message as processed
                let _ = self.mark_message_processed(&message_id).await;
            }
            Message::ChainRegistered { chain_id, timestamp } => {
                // Register a new chain that has the application
                let _ = self.state.subscribed_chains.insert(&chain_id, timestamp);
            }
        }
    }

    async fn store(self) {
        // State is automatically saved by Linera SDK
    }
}

// ============================================================================
// Core Game Logic Implementation (minimal scaffolding)
// ============================================================================

impl PredictionMarketContract {
    /// Initialize the achievement system with predefined achievements
    /// This sets up the reward system for player progression
    async fn initialize_achievements(&mut self) -> Result<(), ContractError> {
        let achievements = vec![
            Achievement {
                id: 1,
                name: "Market Creator".to_string(),
                description: "Create your first market".to_string(),
                reward_tokens: Amount::from_tokens(100),
                reward_xp: 200,
                requirement: AchievementRequirement::CreateMarket,
            },
            Achievement {
                id: 2,
                name: "First Buyer".to_string(),
                description: "Make your first purchase".to_string(),
                reward_tokens: Amount::from_tokens(50),
                reward_xp: 100,
                requirement: AchievementRequirement::FirstBuy,
            },
            Achievement {
                id: 3,
                name: "First Seller".to_string(),
                description: "Make your first sale".to_string(),
                reward_tokens: Amount::from_tokens(50),
                reward_xp: 100,
                requirement: AchievementRequirement::FirstSell,
            },
            Achievement {
                id: 4,
                name: "Guild Member".to_string(),
                description: "Join a guild".to_string(),
                reward_tokens: Amount::from_tokens(150),
                reward_xp: 300,
                requirement: AchievementRequirement::JoinGuild,
            },
            Achievement {
                id: 5,
                name: "Level 2 Achiever".to_string(),
                description: "Reach level 2".to_string(),
                reward_tokens: Amount::from_tokens(200),
                reward_xp: 400,
                requirement: AchievementRequirement::ReachLevel(2),
            },
            Achievement {
                id: 6,
                name: "Level 3 Achiever".to_string(),
                description: "Reach level 3".to_string(),
                reward_tokens: Amount::from_tokens(400),
                reward_xp: 800,
                requirement: AchievementRequirement::ReachLevel(3),
            },
            Achievement {
                id: 7,
                name: "Level 5 Achiever".to_string(),
                description: "Reach level 5".to_string(),
                reward_tokens: Amount::from_tokens(1000),
                reward_xp: 2000,
                requirement: AchievementRequirement::ReachLevel(5),
            },
        ];

        for achievement in achievements {
            self.state
                .achievements
                .insert(&achievement.id.clone(), achievement)?;
        }
        Ok(())
    }

    /// Register a new player in the prediction market game
    /// Creates a player account with initial tokens and sets up their profile
    ///
    /// # Arguments
    /// * `player_id` - The unique identifier for the player
    /// * `display_name` - Optional display name for the player
    /// * `current_time` - Current timestamp for registration
    ///
    /// # Returns
    /// * `Ok(())` - Player successfully registered
    /// * `Err(PlayerAlreadyExists)` - Player already exists
    async fn register_player(
        &mut self,
        player_id: PlayerId,
        display_name: Option<String>,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        if self.state.players.contains_key(&player_id).await? {
            return Err(ContractError::PlayerAlreadyExists);
        }

        let config = self.state.config.get();
        let initial_tokens = config.initial_player_tokens;

        // Give initial points to the player (no external transfer needed)

        let display_name_clone = display_name.clone();
        let player = Player {
            id: player_id,
            display_name,
            registration_time: current_time,
            last_login: current_time,
            token_balance: initial_tokens,
            total_earned: initial_tokens,
            total_spent: Amount::ZERO,
            level: 1,
            experience_points: 0,
            reputation: 100,
            markets_participated: 0,
            markets_won: 0,
            total_profit: Amount::ZERO,
            win_streak: 0,
            best_win_streak: 0,
            guild_id: None,
            achievements_earned: Vec::new(),
            active_markets: Vec::new(),
        };

        self.state.players.insert(&player_id, player)?;

        let total_supply = self.state.total_supply.get().saturating_add(initial_tokens);
        self.state.total_supply.set(total_supply);

        // Broadcast player registration to all chains for horizontal scaling
        self.broadcast_global_player_registered(player_id, display_name_clone).await;

        Ok(())
    }
    /// Update a player's profile information
    /// Allows players to change their display name
    ///
    /// # Arguments
    /// * `player_id` - The player to update
    /// * `display_name` - New display name (can be None to clear)
    ///
    /// # Returns
    /// * `Ok(())` - Profile updated successfully
    /// * `Err(PlayerNotFound)` - Player doesn't exist
    async fn update_player_profile(
        &mut self,
        player_id: PlayerId,
        display_name: Option<String>,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        player.display_name = display_name;
        self.state.players.insert(&player_id, player)?;
        Ok(())
    }

    /// Claim daily login reward for a player
    /// Gives players free tokens for logging in daily (once per 24 hours)
    ///
    /// # Arguments
    /// * `player_id` - The player claiming the reward
    /// * `current_time` - Current timestamp to check 24-hour cooldown
    ///
    /// # Returns
    /// * `Ok(())` - Reward claimed successfully
    /// * `Err(DailyRewardAlreadyClaimed)` - Already claimed within 24 hours
    /// * `Err(PlayerNotFound)` - Player doesn't exist
    async fn claim_daily_reward(
        &mut self,
        player_id: PlayerId,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        let config = self.state.config.get();

        let time_diff = current_time.micros() - player.last_login.micros();
        let one_day_micros = 24 * 60 * 60 * 1_000_000;
        if time_diff < one_day_micros {
            return Err(ContractError::DailyRewardAlreadyClaimed);
        }

        let reward = config.daily_login_reward;

        // Add reward points to the player (no external transfer needed)

        player.token_balance = player.token_balance.saturating_add(reward);
        player.total_earned = player.total_earned.saturating_add(reward);
        player.last_login = current_time;
        self.state.players.insert(&player_id, player)?;

        let total_supply = self.state.total_supply.get();
        let new_total = total_supply.saturating_add(reward);
        self.state.total_supply.set(new_total);
        Ok(())
    }

    /// Create a new point trading market (Market creators must be level 5+ and have 10,000 points)
    /// Only players at level 5 or higher with at least 10,000 points can create markets
    /// Market creation costs 100 points (goes to platform total supply)
    /// Market creators can set their own fee percentage (0-100%) to earn from trades
    ///
    /// # Arguments
    /// * `creator` - The player creating the market (must be level 5+ and have 10,000+ points)
    /// * `title` - Market title/name
    /// * `amount` - Amount of points available in this market
    /// * `fee_percent` - Fee percentage market creator wants to charge on trades (0-100)
    /// * `current_time` - Current timestamp for market timing
    ///
    /// # Returns
    /// * `Ok(())` - Market created successfully
    /// * `Err(InsufficientLevel)` - Player must be at least level 5 to create markets
    /// * `Err(InsufficientBalance)` - Player must have at least 10,000 points to create markets
    async fn create_market(
        &mut self,
        creator: PlayerId,
        title: String,
        amount: Amount,
        fee_percent: u8,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        // Get player - must exist and be registered
        let mut player = self.get_player(&creator).await?;

        // Market creators must be at least level 5
        if player.level < 5 {
            return Err(ContractError::InsufficientLevel);
        }

        // Market creators must have at least 10,000 points
        let min_points_to_create = Amount::from_tokens(10000);
        if player.token_balance < min_points_to_create {
            return Err(ContractError::InsufficientBalance);
        }

        // Market creation costs 100 points
        let creation_cost = Amount::from_tokens(100);
        if player.token_balance < creation_cost {
            return Err(ContractError::InsufficientBalance);
        }

        // Validate fee percentage (0-100)
        if fee_percent > 100 {
            return Err(ContractError::InvalidOutcome); // Reuse error type for now
        }

        // Deduct creation cost from player
        player.token_balance = player.token_balance.saturating_sub(creation_cost);
        player.total_spent = player.total_spent.saturating_add(creation_cost);
        self.state.players.insert(&creator, player)?;

        // Distribute market creation fee to platform (100 points goes to total supply)
        self.distribute_market_creator_fee(creator, creation_cost)
            .await?;

        let market_id = self.generate_market_id().await?;
        let title_clone = title.clone();

        let market = Market {
            id: market_id,
            creator,
            title,
            amount,
            fee_percent,
            creation_time: current_time,
            status: MarketStatus::Active,
            total_liquidity: amount,
            positions: BTreeMap::new(),
            total_participants: 0,
        };

        self.state.markets.insert(&market_id, market)?;

        // Update creator's active markets
        let mut creator_player = self.get_player(&creator).await?;
        if !creator_player.active_markets.contains(&market_id) {
            creator_player.active_markets.push(market_id);
        }
        self.state
            .players
            .insert(&creator, creator_player.clone())?;

        // Check for achievements after creating market
        self.check_achievements(&mut creator_player).await?;

        self.runtime
            .prepare_message(Message::MarketCreated { market_id, creator })
            .send_to(self.runtime.chain_id());

        // Broadcast market creation to all chains for horizontal scaling
        self.broadcast_global_market_created(market_id, creator, title_clone).await;

        Ok(())
    }

    /// Buy points from a market
    /// Allows players to buy points from active markets with level-based progressive exchange rate
    /// Market creator receives fee based on their chosen fee percentage
    /// Exchange rate scales with player level:
    /// - Level 1: pay 10 points to get 100 points (10:1 ratio)
    /// - Level 2: pay 100 points to get 1000 points (10:1 ratio)
    ///
    /// # Arguments
    /// * `player_id` - The player buying points
    /// * `market_id` - The market to buy points from
    /// * `amount` - How many points the player wants to receive
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Points purchased successfully
    /// * `Err(InsufficientBalance)` - Player doesn't have enough tokens to pay
    /// * `Err(MarketNotFound)` - Market doesn't exist
    /// * `Err(MarketNotActive)` - Market is not active
    async fn buy_shares(
        &mut self,
        player_id: PlayerId,
        market_id: MarketId,
        amount: Amount,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;

        // Get the specific market to buy from
        let mut market = self.get_market(&market_id).await?;

        if market.status != MarketStatus::Active {
            return Err(ContractError::MarketNotActive);
        }

        // Check if market has enough liquidity to sell
        let points_to_receive = market.total_liquidity.min(amount);
        if points_to_receive == Amount::ZERO {
            return Err(ContractError::InsufficientBalance);
        }

        // Calculate payment based on progressive exchange (works for all levels 1 to infinity)
        // Exchange rate is always 10:1 (pay 10% to receive 100%)
        // Examples:
        //   Level 1: want 100 points → pay 10 points
        //   Level 2: want 1000 points → pay 100 points
        //   Level 3: want 10000 points → pay 1000 points
        //   Level N: want X points → pay X/10 points
        let points_tokens: u128 = points_to_receive.into();
        let base_payment_tokens = points_tokens.saturating_div(10);
        let base_payment = Amount::from_tokens(base_payment_tokens);

        // Calculate fee based on market's fee percentage
        let fee_divisor = 100_u128;
        let fee_tokens = base_payment_tokens
            .saturating_mul(market.fee_percent as u128)
            .saturating_div(fee_divisor);
        let buyer_fee = Amount::from_tokens(fee_tokens);
        let actual_payment = base_payment.saturating_add(buyer_fee);

        // Ensure player has enough to pay (base payment + fee)
        if player.token_balance < actual_payment {
            return Err(ContractError::InsufficientBalance);
        }

        // Player pays base payment + fee
        player.token_balance = player.token_balance.saturating_sub(actual_payment);
        player.total_spent = player.total_spent.saturating_add(actual_payment);

        // Player receives points from the market (transfer from market liquidity to player)
        player.token_balance = player.token_balance.saturating_add(points_to_receive);
        player.total_earned = player.total_earned.saturating_add(points_to_receive);

        // Distribute trading fees: creator gets (base payment + fee), platform gets 2% of creator's fee
        // First, give market creator the base payment
        let mut market_creator = self.get_player(&market.creator).await?;
        market_creator.token_balance = market_creator.token_balance.saturating_add(base_payment);
        market_creator.total_earned = market_creator.total_earned.saturating_add(base_payment);
        self.state.players.insert(&market.creator, market_creator)?;

        // Distribute the fee portion: creator keeps 98%, platform gets 2%
        self.distribute_trading_fees(market_id, buyer_fee).await?;

        // Update market liquidity (points available decrease as they're sold)
        market.total_liquidity = market.total_liquidity.saturating_sub(points_to_receive);

        // Update position
        let position = market.positions.entry(player_id).or_insert(PlayerPosition {
            shares_by_outcome: BTreeMap::new(),
            total_invested: Amount::ZERO,
            entry_time: current_time,
        });
        position.total_invested = position.total_invested.saturating_add(actual_payment);

        if !player.active_markets.contains(&market_id) {
            player.active_markets.push(market_id);
            market.total_participants += 1;
        }

        player.markets_participated += 1;
        self.add_experience(&mut player, 10).await?;

        self.state.markets.insert(&market_id, market)?;
        self.state.players.insert(&player_id, player.clone())?;

        // Check for achievements after buying (first buy achievement)
        self.check_achievements(&mut player).await?;

        self.runtime
            .prepare_message(Message::TradeExecuted {
                player_id,
                market_id,
                outcome_id: 0,
                shares: points_to_receive,
                price: base_payment,
            })
            .send_to(self.runtime.chain_id());
        Ok(())
    }

    /// Sell points to a market
    /// Allows players at level 5+ to sell their points to a specific market
    /// Market creator receives fee based on their chosen fee percentage
    ///
    /// # Arguments
    /// * `player_id` - The player selling points
    /// * `market_id` - The market to sell points to
    /// * `amount` - How many points to sell
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Points sold successfully
    /// * `Err(InsufficientLevel)` - Player must be at least level 5 to sell
    /// * `Err(InsufficientBalance)` - Player doesn't have enough points to sell
    /// * `Err(MarketNotFound)` - Market doesn't exist
    /// * `Err(MarketNotActive)` - Market is not active
    async fn sell_shares(
        &mut self,
        player_id: PlayerId,
        market_id: MarketId,
        amount: Amount,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;

        // Sellers must be at least level 5
        if player.level < 5 {
            return Err(ContractError::InsufficientLevel);
        }

        if player.token_balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        // Get the specific market to sell to
        let mut market = self.get_market(&market_id).await?;

        if market.status != MarketStatus::Active {
            return Err(ContractError::MarketNotActive);
        }

        // Calculate seller fee using the market's fee percentage (set by market creator)
        let fee_divisor = 100_u128;
        let amount_tokens: u128 = amount.into();
        let fee_tokens = amount_tokens
            .saturating_mul(market.fee_percent as u128)
            .saturating_div(fee_divisor);
        let seller_fee = Amount::from_tokens(fee_tokens);
        let points_for_market = amount.saturating_sub(seller_fee);

        // Burn points from player (decrease their balance)
        player.token_balance = player.token_balance.saturating_sub(amount);
        player.total_spent = player.total_spent.saturating_add(amount);

        // Distribute trading fees: creator gets 98% of fee, platform gets 2% of fee
        self.distribute_trading_fees(market_id, seller_fee).await?;

        // Burn points from total supply (net after fee)
        let current_supply = self.state.total_supply.get();
        self.state
            .total_supply
            .set(current_supply.saturating_sub(points_for_market.min(*current_supply)));

        // Add liquidity to market (points available increase, minus fee)
        market.total_liquidity = market.total_liquidity.saturating_add(points_for_market);

        // Update position
        let position = market.positions.entry(player_id).or_insert(PlayerPosition {
            shares_by_outcome: BTreeMap::new(),
            total_invested: Amount::ZERO,
            entry_time: current_time,
        });
        position.total_invested = position.total_invested.saturating_add(amount);

        if !player.active_markets.contains(&market_id) {
            player.active_markets.push(market_id);
            market.total_participants += 1;
        }

        player.markets_participated += 1;
        self.add_experience(&mut player, 10).await?;

        self.state.markets.insert(&market_id, market)?;
        self.state.players.insert(&player_id, player.clone())?;

        // Check for achievements after selling (first sell achievement)
        self.check_achievements(&mut player).await?;

        self.runtime
            .prepare_message(Message::TradeExecuted {
                player_id,
                market_id,
                outcome_id: 0,
                shares: amount,
                price: amount,
            })
            .send_to(self.runtime.chain_id());
        Ok(())
    }

    /// Mint points (Admin only)
    /// Allows admin to mint more points to the total supply
    ///
    /// # Arguments
    /// * `caller` - The player attempting to mint points
    /// * `amount` - How many points to mint
    ///
    /// # Returns
    /// * `Ok(())` - Points minted successfully
    /// * `Err(NotAdmin)` - Caller is not the admin
    async fn mint_points(&mut self, caller: PlayerId, amount: Amount) -> Result<(), ContractError> {
        let config = self.state.config.get();

        // Only admin can mint points
        if let Some(admin) = config.admin {
            if caller != admin {
                return Err(ContractError::NotAdmin);
            }
        } else {
            return Err(ContractError::NotAdmin);
        }

        // Increase total supply
        let current_supply = self.state.total_supply.get();
        self.state
            .total_supply
            .set(current_supply.saturating_add(amount));

        Ok(())
    }

    /// Create a new guild
    /// Allows players to form social groups for collaborative gameplay
    ///
    /// # Arguments
    /// * `founder` - The player creating the guild
    /// * `name` - The name of the guild
    /// * `current_time` - Current timestamp for guild creation
    ///
    /// # Returns
    /// * `Ok(())` - Guild created successfully
    /// * `Err(AlreadyInGuild)` - Founder is already in a guild
    async fn create_guild(
        &mut self,
        founder: PlayerId,
        name: String,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&founder).await?;
        if player.guild_id.is_some() {
            return Err(ContractError::AlreadyInGuild);
        }
        let new_id = self.next_guild_id().await?;
        let guild = Guild {
            id: new_id,
            name: name.clone(),
            founder,
            members: vec![founder],
            creation_time: current_time,
            total_guild_profit: Amount::ZERO,
            guild_level: 1,
            shared_pool: Amount::ZERO,
        };
        self.state.guilds.insert(&new_id, guild)?;
        player.guild_id = Some(new_id);
        self.state.players.insert(&founder, player)?;

        self.runtime
            .prepare_message(Message::GuildCreated {
                guild_id: new_id,
                name: name.clone(),
            })
            .send_to(self.runtime.chain_id());

        // Broadcast guild creation to all chains for horizontal scaling
        self.broadcast_global_guild_created(new_id, name, founder).await;
        Ok(())
    }

    /// Join an existing guild
    /// Allows players to join guilds created by other players
    ///
    /// # Arguments
    /// * `player_id` - The player joining the guild
    /// * `guild_id` - The guild to join
    ///
    /// # Returns
    /// * `Ok(())` - Successfully joined guild
    /// * `Err(AlreadyInGuild)` - Player is already in a guild
    /// * `Err(GuildNotFound)` - Guild doesn't exist
    async fn join_guild(
        &mut self,
        player_id: PlayerId,
        guild_id: GuildId,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        if player.guild_id.is_some() {
            return Err(ContractError::AlreadyInGuild);
        }
        let mut guild = self
            .state
            .guilds
            .get(&guild_id)
            .await?
            .ok_or(ContractError::GuildNotFound)?;
        guild.members.push(player_id);
        self.state.guilds.insert(&guild_id, guild)?;
        player.guild_id = Some(guild_id);
        self.state.players.insert(&player_id, player.clone())?;

        // Check for achievements after joining guild
        self.check_achievements(&mut player).await?;

        Ok(())
    }

    /// Leave the current guild
    /// Allows players to leave their current guild
    ///
    /// # Arguments
    /// * `player_id` - The player leaving the guild
    ///
    /// # Returns
    /// * `Ok(())` - Successfully left guild
    /// * `Err(NotGuildMember)` - Player is not in a guild
    async fn leave_guild(&mut self, player_id: PlayerId) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        let guild_id = player.guild_id.ok_or(ContractError::NotGuildMember)?;
        let mut guild = self
            .state
            .guilds
            .get(&guild_id)
            .await?
            .ok_or(ContractError::GuildNotFound)?;
        guild.members.retain(|m| m != &player_id);
        self.state.guilds.insert(&guild_id, guild)?;
        player.guild_id = None;
        self.state.players.insert(&player_id, player)?;
        Ok(())
    }

    /// Contribute tokens to the guild's shared pool
    /// Allows guild members to contribute tokens to the guild's collective fund
    ///
    /// # Arguments
    /// * `player_id` - The player contributing tokens
    /// * `amount` - How many tokens to contribute
    ///
    /// # Returns
    /// * `Ok(())` - Contribution successful
    /// * `Err(NotGuildMember)` - Player is not in a guild
    /// * `Err(InsufficientBalance)` - Player doesn't have enough tokens
    async fn contribute_to_guild(
        &mut self,
        player_id: PlayerId,
        amount: Amount,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        let guild_id = player.guild_id.ok_or(ContractError::NotGuildMember)?;
        if player.token_balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        // Deduct contribution from player's points (no external transfer needed)

        let mut guild = self
            .state
            .guilds
            .get(&guild_id)
            .await?
            .ok_or(ContractError::GuildNotFound)?;
        player.token_balance = player.token_balance.saturating_sub(amount);
        guild.shared_pool = guild.shared_pool.saturating_add(amount);
        self.state.players.insert(&player_id, player)?;
        self.state.guilds.insert(&guild_id, guild)?;
        Ok(())
    }

    /// Update the game configuration (Admin only)
    /// Allows the admin to modify game parameters like token amounts and market settings
    ///
    /// # Arguments
    /// * `caller` - The player attempting to update config
    /// * `config` - The new game configuration
    ///
    /// # Returns
    /// * `Ok(())` - Configuration updated successfully
    /// * `Err(NotAdmin)` - Caller is not the admin
    async fn update_game_config(
        &mut self,
        caller: PlayerId,
        config: GameConfig,
    ) -> Result<(), ContractError> {
        let current = self.state.config.get();
        if let Some(admin) = current.admin {
            if caller != admin {
                return Err(ContractError::NotAdmin);
            }
        } else {
            return Err(ContractError::NotAdmin);
        }
        self.state.config.set(config);
        Ok(())
    }

    // ============================================================================
    // Single-File Prediction Market Game
    // ============================================================================

    /// This contract implements a complete prediction market game with:
    /// - Player progression system (levels, reputation, achievements)
    /// - Market operations (create, trade, resolve)
    /// - Guild system (social features, shared pools)
    /// - Points-based economy (no external tokens needed)
    /// - Admin controls (game configuration)

    // ============================================================================
    // Enhanced Leaderboard System
    // ============================================================================

    /// Update the enhanced leaderboard with sophisticated ranking algorithms
    /// Players are ranked by total points earned (total_earned)
    /// Guilds are ranked by total points earned by all guild members
    async fn update_enhanced_leaderboard(&mut self) {
        let mut top_traders = Vec::new();
        let mut top_guilds = Vec::new();

        // Collect all players and rank by total points earned
        let mut player_scores = Vec::new();
        self.state
            .players
            .for_each_index_value(|player_id, player| {
                let player = player.into_owned();
                // Use total_earned (total points earned) for ranking
                let total_points: u128 = player.total_earned.into();
                player_scores.push((player_id, player, total_points));
                Ok(())
            })
            .await
            .expect("Failed to iterate players");

        // Sort by total points earned (descending)
        player_scores.sort_by(|a, b| b.2.cmp(&a.2));

        // Take top 50 players by total points earned
        for (player_id, player, _total_points) in player_scores.into_iter().take(50) {
            let win_rate = if player.markets_participated > 0 {
                (player.markets_won as f64 / player.markets_participated as f64) * 100.0
            } else {
                0.0
            };

            top_traders.push(LeaderboardEntry {
                player_id,
                display_name: player.display_name,
                total_profit: player.total_earned, // Use total_earned instead of total_profit
                win_rate,
                level: player.level,
            });
        }

        // Collect all guilds and calculate total points earned by all members
        let mut guild_scores = Vec::new();
        self.state
            .guilds
            .for_each_index_value(|guild_id, guild| {
                let guild = guild.into_owned();

                // Calculate total points earned by all guild members
                // We need to collect this separately due to async constraints
                let guild_members = guild.members.clone();
                let guild_name = guild.name.clone();
                guild_scores.push((guild_id, guild_members, guild_name));
                Ok(())
            })
            .await
            .expect("Failed to iterate guilds");

        // Calculate total points for each guild
        let mut guild_totals = Vec::new();
        for (guild_id, members, name) in guild_scores {
            let mut total_guild_points = Amount::ZERO;
            let member_count = members.len() as u32;

            // Sum total points earned by all guild members
            for member_id in &members {
                if let Some(member) = self.state.players.get(member_id).await.ok().flatten() {
                    let member = member.clone();
                    total_guild_points = total_guild_points.saturating_add(member.total_earned);
                }
            }

            let total_points: u128 = total_guild_points.into();
            guild_totals.push((
                guild_id,
                name,
                total_guild_points,
                member_count,
                total_points,
            ));
        }

        // Sort by total points earned by guild (descending)
        guild_totals.sort_by(|a, b| b.4.cmp(&a.4));

        // Take top 20 guilds by total points earned
        for (guild_id, name, total_guild_points, member_count, _total_points) in
            guild_totals.into_iter().take(20)
        {
            top_guilds.push(GuildLeaderboardEntry {
                guild_id,
                name,
                total_profit: total_guild_points, // Use total points earned instead of total_guild_profit
                member_count,
            });
        }

        // Update leaderboard
        let mut leaderboard = self.state.leaderboard.get().clone();
        leaderboard.top_traders = top_traders;
        leaderboard.top_guilds = top_guilds;
        leaderboard.last_updated = self.runtime.system_time();
        self.state.leaderboard.set(leaderboard);
    }

    // ============================================================================
    // Market Creator Fee Distribution
    // ============================================================================

    /// Distribute market creation fee to platform
    /// Market creation costs 100 points, which all go to platform total supply
    async fn distribute_market_creator_fee(
        &mut self,
        _creator: PlayerId,
        creation_fee: Amount,
    ) -> Result<(), ContractError> {
        // Market creation fee (100 points) goes entirely to platform total supply
        let current_supply = self.state.total_supply.get();
        self.state
            .total_supply
            .set(current_supply.saturating_add(creation_fee));

        // Update leaderboard after fee distribution
        self.update_enhanced_leaderboard().await;

        Ok(())
    }

    /// Distribute trading fees to market creator and platform
    /// Platform gets 2% of the creator's fee from each trade
    /// Creator gets the remaining fee (98% of their fee percentage)
    async fn distribute_trading_fees(
        &mut self,
        market_id: MarketId,
        creator_fee_amount: Amount,
    ) -> Result<(), ContractError> {
        let market = self.get_market(&market_id).await?;

        // Platform gets 2% of the creator's fee
        let platform_fee_percent = 2_u128; // 2% of creator's fee
        let fee_divisor = 100_u128;

        let creator_fee_tokens: u128 = creator_fee_amount.into();
        let platform_fee_tokens = creator_fee_tokens
            .saturating_mul(platform_fee_percent)
            .saturating_div(fee_divisor);
        let platform_fee = Amount::from_tokens(platform_fee_tokens);
        let creator_keeps = creator_fee_amount.saturating_sub(platform_fee);

        // Give creator their share (creator's fee minus 2% platform fee)
        let mut creator_player = self.get_player(&market.creator).await?;
        creator_player.token_balance = creator_player.token_balance.saturating_add(creator_keeps);
        creator_player.total_earned = creator_player.total_earned.saturating_add(creator_keeps);
        self.state.players.insert(&market.creator, creator_player)?;

        // Add platform fee (2% of creator's fee) to total supply
        let current_supply = self.state.total_supply.get();
        self.state
            .total_supply
            .set(current_supply.saturating_add(platform_fee));

        Ok(())
    }

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Get a player by their ID
    /// Helper function to retrieve player data from storage
    async fn get_player(&self, player_id: &PlayerId) -> Result<Player, ContractError> {
        self.state
            .players
            .get(player_id)
            .await?
            .ok_or(ContractError::PlayerNotFound)
    }

    /// Get a market by its ID
    /// Helper function to retrieve market data from storage
    async fn get_market(&self, market_id: &MarketId) -> Result<Market, ContractError> {
        self.state
            .markets
            .get(market_id)
            .await?
            .ok_or(ContractError::MarketNotFound)
    }

    /// Generate a unique market ID
    /// Helper function to create unique IDs for new markets
    async fn generate_market_id(&mut self) -> Result<MarketId, ContractError> {
        let id = *self.state.next_market_id.get();
        let new_id = id + 1;
        self.state.next_market_id.set(new_id);
        Ok(id)
    }

    /// Generate a unique guild ID
    /// Helper function to create unique IDs for new guilds
    async fn next_guild_id(&mut self) -> Result<GuildId, ContractError> {
        // naive: number of guilds as next id
        // MapView has no len; use timestamp lower bits for uniqueness
        Ok((self.runtime.system_time().micros() & 0xFFFF_FFFF) as u64)
    }

    /// Add experience points to a player and handle leveling up
    /// Leveling formula:
    /// - Level 1: needs 1000 total points
    /// - Level 2: needs 4000 total points (Level 1 max * 4)
    /// - Level 3: needs 16000 total points (Level 2 max * 4)
    /// - Formula: total_points_needed = 1000 * (4^(level-1))
    /// As levels increase, players get more points per action, making it easier to progress
    async fn add_experience(&mut self, player: &mut Player, xp: u64) -> Result<(), ContractError> {
        player.experience_points += xp;
        let old_level = player.level;

        // New leveling formula based on total experience points
        // Level 1: needs 1000 total points
        // Level 2: needs 4000 total points (1000 * 4)
        // Level 3: needs 16000 total points (4000 * 4)
        // Formula: total_points_for_level(N) = 1000 * (4^(N-1))
        while player.level >= 1 {
            let current_level = player.level as u64;
            let total_points_needed = if current_level == 1 {
                1000 // Level 1 needs 1000 total points
            } else {
                1000_u64 * 4_u64.pow((current_level - 1) as u32) // Level N needs 1000 * 4^(N-1)
            };

            // Check if player has enough total points for next level
            if player.experience_points >= total_points_needed {
                player.level += 1;
                // Don't subtract - we track total experience points, not incremental
                // Continue checking if they can level up multiple times
            } else {
                break; // Not enough points for next level
            }
        }

        // Check for level-based achievements when leveling up
        if player.level > old_level {
            self.check_achievements(player).await?;
        }

        Ok(())
    }

    /// Check and award achievements for a player
    async fn check_achievements(&mut self, player: &mut Player) -> Result<(), ContractError> {
        let mut new_achievements = Vec::new();

        // Check all achievements
        for achievement_id in 1..=7 {
            if let Some(achievement) = self.state.achievements.get(&achievement_id).await? {
                if !player.achievements_earned.contains(&achievement_id) {
                    if self
                        .check_achievement_requirement(player, &achievement.requirement)
                        .await?
                    {
                        // Award achievement
                        player.achievements_earned.push(achievement_id);
                        player.token_balance = player
                            .token_balance
                            .saturating_add(achievement.reward_tokens);
                        player.total_earned = player
                            .total_earned
                            .saturating_add(achievement.reward_tokens);
                        player.experience_points += achievement.reward_xp;

                        new_achievements.push(achievement_id);

                        // Send achievement notification
                        self.runtime
                            .prepare_message(Message::AchievementUnlocked {
                                player_id: player.id,
                                achievement_id,
                            })
                            .send_to(self.runtime.chain_id());
                    }
                }
            }
        }

        // Update player with new achievements
        if !new_achievements.is_empty() {
            self.state.players.insert(&player.id, player.clone())?;
        }

        Ok(())
    }

    /// Check if a player meets an achievement requirement
    async fn check_achievement_requirement(
        &self,
        player: &Player,
        requirement: &AchievementRequirement,
    ) -> Result<bool, ContractError> {
        match requirement {
            // New achievement requirements
            AchievementRequirement::CreateMarket => {
                // Count markets created by this player
                let mut created_count = 0;
                for market_id in &player.active_markets {
                    if let Some(market) = self.state.markets.get(market_id).await? {
                        if market.creator == player.id {
                            created_count += 1;
                        }
                    }
                }
                Ok(created_count >= 1)
            }
            AchievementRequirement::FirstBuy => {
                // Check if player has made at least one buy
                // Player has participated in markets (bought points from a market)
                Ok(player.markets_participated > 0)
            }
            AchievementRequirement::FirstSell => {
                // Check if player has sold points
                // A player has sold if they've spent points (sold points reduce balance)
                // We can also check if they've participated in markets as a proxy
                Ok(player.total_spent > Amount::ZERO)
            }
            AchievementRequirement::JoinGuild => Ok(player.guild_id.is_some()),
            AchievementRequirement::ReachLevel(level) => Ok(player.level >= *level),
            // Legacy requirements (kept for backward compatibility)
            AchievementRequirement::WinMarkets(count) => Ok(player.markets_won >= *count),
            AchievementRequirement::WinStreak(streak) => Ok(player.win_streak >= *streak),
            AchievementRequirement::TotalProfit(profit) => Ok(player.total_profit >= *profit),
            AchievementRequirement::ParticipateInMarkets(count) => {
                Ok(player.markets_participated >= *count)
            }
            AchievementRequirement::CreateMarkets(count) => {
                // Count markets created by this player
                let mut created_count = 0;
                for market_id in &player.active_markets {
                    if let Some(market) = self.state.markets.get(market_id).await? {
                        if market.creator == player.id {
                            created_count += 1;
                        }
                    }
                }
                Ok(created_count >= *count)
            }
        }
    }

    // ============================================================================
    //  Price Prediction Functions
    // ============================================================================

    /// The price prediction model predicts the price of real assets market price from crypto price API providers
    /// like CoinMarketCap, CoinGecko, etc.
    /// Players can predict if the market will rise or fall daily, weekly and monthly and earn point tokens
    /// based on the accuracy of the prediction.
    ///
    /// IMPORTANT: All price comparisons use data from crypto API providers:
    /// - Oracle/Admin calls update_market_price() with price from crypto API (CoinMarketCap, CoinGecko, etc.)
    /// - Initial price is captured from crypto API when period starts
    /// - End price is captured from crypto API when period ends
    /// - Player's prediction is compared against actual outcome from crypto API price changes
    ///
    /// The outcome will depend on the change of crypto API price:
    /// - Rise: end_price > initial_price (crypto API price increased from initial)
    /// - Fall: end_price < initial_price (crypto API price decreased from initial)
    /// - Neutral: end_price == initial_price (crypto API price stayed the same as initial)

    /// Make a daily prediction for market price movement
    /// Players predict if the market price will rise, fall, or stay neutral over the next 24 hours
    ///
    /// # Arguments
    /// * `player_id` - The player making the prediction
    /// * `outcome` - The predicted outcome (Rise, Fall, or Neutral)
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Prediction recorded successfully
    /// * `Err(PlayerNotFound)` - Player doesn't exist
    async fn predict_daily_outcome(
        &mut self,
        player_id: PlayerId,
        outcome: PriceOutcome,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let _player = self.get_player(&player_id).await?;

        // Calculate the current day period start (midnight of current day)
        let period_start = self.get_daily_period_start(current_time);

        // Check if player already has a prediction for this period
        let prediction_key = format!(
            "{:?}_{:?}_{}",
            player_id,
            PredictionPeriod::Daily,
            period_start.micros()
        );

        if self.state.predictions.contains_key(&prediction_key).await? {
            return Err(ContractError::InvalidOutcome); // Reuse error: already predicted for this period
        }

        // Create the prediction
        let prediction = PlayerPrediction {
            player_id,
            period: PredictionPeriod::Daily,
            outcome,
            prediction_time: current_time,
            period_start,
            resolved: false,
            correct: None,
        };

        // Store the prediction
        self.state
            .predictions
            .insert(&prediction_key, prediction.clone())?;

        // Initialize period price data if it doesn't exist
        let period_key = format!("{:?}_{}", PredictionPeriod::Daily, period_start.micros());
        if !self.state.period_prices.contains_key(&period_key).await? {
            // Capture the initial market price from crypto API when the period starts
            // This price comes from crypto API providers (CoinMarketCap, CoinGecko, etc.)
            // via the update_market_price function called by oracle/admin
            let initial_price = self.state.current_market_price.get();
            let period_price_data = PeriodPriceData {
                period_start,
                period_end: Timestamp::from(
                    period_start
                        .micros()
                        .saturating_add(24 * 60 * 60 * 1_000_000),
                ), // 24 hours later
                start_price: Some(initial_price.clone()), // Initial crypto API price at period start
                end_price: None, // Will be set from crypto API when period ends
                outcome: None,
                resolved: false,
            };
            self.state
                .period_prices
                .insert(&period_key, period_price_data)?;
        }

        // Send prediction message
        self.runtime
            .prepare_message(Message::PredictionMade {
                player_id,
                period: PredictionPeriod::Daily,
                outcome,
            })
            .send_to(self.runtime.chain_id());

        Ok(())
    }

    /// Make a weekly prediction for market price movement
    /// Players predict if the market price will rise, fall, or stay neutral over the next 7 days
    ///
    /// # Arguments
    /// * `player_id` - The player making the prediction
    /// * `outcome` - The predicted outcome (Rise, Fall, or Neutral)
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Prediction recorded successfully
    /// * `Err(PlayerNotFound)` - Player doesn't exist
    async fn predict_weekly_outcome(
        &mut self,
        player_id: PlayerId,
        outcome: PriceOutcome,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let _player = self.get_player(&player_id).await?;

        // Calculate the current week period start (start of current week)
        let period_start = self.get_weekly_period_start(current_time);

        // Check if player already has a prediction for this period
        let prediction_key = format!(
            "{:?}_{:?}_{}",
            player_id,
            PredictionPeriod::Weekly,
            period_start.micros()
        );

        if self.state.predictions.contains_key(&prediction_key).await? {
            return Err(ContractError::InvalidOutcome); // Already predicted for this period
        }

        // Create the prediction
        let prediction = PlayerPrediction {
            player_id,
            period: PredictionPeriod::Weekly,
            outcome,
            prediction_time: current_time,
            period_start,
            resolved: false,
            correct: None,
        };

        // Store the prediction
        self.state
            .predictions
            .insert(&prediction_key, prediction.clone())?;

        // Initialize period price data if it doesn't exist
        let period_key = format!("{:?}_{}", PredictionPeriod::Weekly, period_start.micros());
        if !self.state.period_prices.contains_key(&period_key).await? {
            // Capture the initial market price from crypto API when the period starts
            // This price comes from crypto API providers (CoinMarketCap, CoinGecko, etc.)
            // via the update_market_price function called by oracle/admin
            let initial_price = self.state.current_market_price.get();
            let period_price_data = PeriodPriceData {
                period_start,
                period_end: Timestamp::from(
                    period_start
                        .micros()
                        .saturating_add(7 * 24 * 60 * 60 * 1_000_000),
                ), // 7 days later
                start_price: Some(initial_price.clone()), // Initial crypto API price at period start
                end_price: None, // Will be set from crypto API when period ends
                outcome: None,
                resolved: false,
            };
            self.state
                .period_prices
                .insert(&period_key, period_price_data)?;
        }

        // Send prediction message
        self.runtime
            .prepare_message(Message::PredictionMade {
                player_id,
                period: PredictionPeriod::Weekly,
                outcome,
            })
            .send_to(self.runtime.chain_id());

        Ok(())
    }

    /// Make a monthly prediction for market price movement
    /// Players predict if the market price will rise, fall, or stay neutral over the next 30 days
    ///
    /// # Arguments
    /// * `player_id` - The player making the prediction
    /// * `outcome` - The predicted outcome (Rise, Fall, or Neutral)
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Prediction recorded successfully
    /// * `Err(PlayerNotFound)` - Player doesn't exist
    async fn predict_monthly_outcome(
        &mut self,
        player_id: PlayerId,
        outcome: PriceOutcome,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let _player = self.get_player(&player_id).await?;

        // Calculate the current month period start (start of current month)
        let period_start = self.get_monthly_period_start(current_time);

        // Check if player already has a prediction for this period
        let prediction_key = format!(
            "{:?}_{:?}_{}",
            player_id,
            PredictionPeriod::Monthly,
            period_start.micros()
        );

        if self.state.predictions.contains_key(&prediction_key).await? {
            return Err(ContractError::InvalidOutcome); // Already predicted for this period
        }

        // Create the prediction
        let prediction = PlayerPrediction {
            player_id,
            period: PredictionPeriod::Monthly,
            outcome,
            prediction_time: current_time,
            period_start,
            resolved: false,
            correct: None,
        };

        // Store the prediction
        self.state
            .predictions
            .insert(&prediction_key, prediction.clone())?;

        // Initialize period price data if it doesn't exist
        let period_key = format!("{:?}_{}", PredictionPeriod::Monthly, period_start.micros());
        if !self.state.period_prices.contains_key(&period_key).await? {
            // Capture the initial market price from crypto API when the period starts
            // This price comes from crypto API providers (CoinMarketCap, CoinGecko, etc.)
            // via the update_market_price function called by oracle/admin
            let initial_price = self.state.current_market_price.get();
            let period_price_data = PeriodPriceData {
                period_start,
                period_end: Timestamp::from(
                    period_start
                        .micros()
                        .saturating_add(30 * 24 * 60 * 60 * 1_000_000),
                ), // 30 days later
                start_price: Some(initial_price.clone()), // Initial crypto API price at period start
                end_price: None, // Will be set from crypto API when period ends
                outcome: None,
                resolved: false,
            };
            self.state
                .period_prices
                .insert(&period_key, period_price_data)?;
        }

        // Send prediction message
        self.runtime
            .prepare_message(Message::PredictionMade {
                player_id,
                period: PredictionPeriod::Monthly,
                outcome,
            })
            .send_to(self.runtime.chain_id());

        Ok(())
    }

    /// Update the current market price from crypto price API providers (Oracle/Admin only)
    /// This function is called by an oracle or admin to update market prices from external APIs
    /// like CoinMarketCap, CoinGecko, etc.
    /// When prices are updated, the contract automatically resolves predictions for expired periods
    ///
    /// # Arguments
    /// * `caller` - The player calling this function (must be admin/oracle)
    /// * `price` - The current market price from crypto API provider (e.g., CoinMarketCap, CoinGecko)
    /// * `current_time` - Current timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Price updated successfully
    /// * `Err(NotAdmin)` - Caller is not authorized
    ///
    /// # Usage Example:
    /// ```rust
    /// // Oracle fetches price from CoinGecko API (e.g., BTC price = $50,000)
    /// let crypto_price = Amount::from_tokens(50000); // Convert to Amount
    /// contract.update_market_price(admin_id, crypto_price, current_time).await?;
    /// ```
    async fn update_market_price(
        &mut self,
        caller: PlayerId,
        price: Amount,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        let config = self.state.config.get();

        // Only admin can update market prices (oracle functionality)
        // In production, this would be called by an oracle service that fetches
        // prices from crypto API providers like CoinMarketCap, CoinGecko, etc.
        if let Some(admin) = config.admin {
            if caller != admin {
                return Err(ContractError::NotAdmin);
            }
        } else {
            return Err(ContractError::NotAdmin);
        }

        // Update current market price with data from crypto API provider
        let market_price = MarketPrice {
            price, // Price from crypto API (CoinMarketCap, CoinGecko, etc.)
            timestamp: current_time,
        };
        self.state.current_market_price.set(market_price);

        // Broadcast price update to all chains for horizontal scaling
        self.broadcast_global_price_update(price, current_time).await;

        // Try to resolve expired periods and their predictions
        // This compares end_price (from crypto API) to initial_price (from crypto API)
        self.resolve_expired_predictions(current_time).await?;

        Ok(())
    }

    // ============================================================================
    // Price Prediction Helper Functions
    // ============================================================================

    /// Calculate the start timestamp for a daily period (midnight of the current day)
    fn get_daily_period_start(&self, timestamp: Timestamp) -> Timestamp {
        // For simplicity, use current timestamp rounded down to day boundary
        // In production, this should properly calculate midnight
        let one_day_micros = 24 * 60 * 60 * 1_000_000;
        let day_start = (timestamp.micros() / one_day_micros) * one_day_micros;
        Timestamp::from(day_start)
    }

    /// Calculate the start timestamp for a weekly period (start of current week)
    fn get_weekly_period_start(&self, timestamp: Timestamp) -> Timestamp {
        // For simplicity, use current timestamp rounded down to week boundary
        // In production, this should properly calculate week start (e.g., Monday)
        let one_week_micros = 7 * 24 * 60 * 60 * 1_000_000;
        let week_start = (timestamp.micros() / one_week_micros) * one_week_micros;
        Timestamp::from(week_start)
    }

    /// Calculate the start timestamp for a monthly period (start of current month)
    fn get_monthly_period_start(&self, timestamp: Timestamp) -> Timestamp {
        // For simplicity, use current timestamp rounded down to month boundary
        // In production, this should properly calculate month start
        let one_month_micros = 30 * 24 * 60 * 60 * 1_000_000; // Approximate
        let month_start = (timestamp.micros() / one_month_micros) * one_month_micros;
        Timestamp::from(month_start)
    }

    /// Resolve a prediction by comparing it to the actual outcome
    /// This function calculates whether a player's prediction was correct
    async fn resolve_prediction(
        &mut self,
        prediction: &mut PlayerPrediction,
        period: PredictionPeriod,
        period_start: Timestamp,
    ) -> Result<(), ContractError> {
        if prediction.resolved {
            return Ok(()); // Already resolved
        }

        // Get period price data
        let period_key = format!("{:?}_{}", period, period_start.micros());

        if let Some(mut period_data) = self
            .state
            .period_prices
            .get(&period_key)
            .await?
            .map(|p| p.clone())
        {
            // Check if period has both initial price (start_price) and end price from crypto API
            if let (Some(initial_price), Some(end_price)) =
                (&period_data.start_price, &period_data.end_price)
            {
                // Calculate the actual outcome: compare end_price (from crypto API) to initial_price (from crypto API)
                // Both prices come from crypto API providers (CoinMarketCap, CoinGecko, etc.)
                // Rise: end_price > initial_price (crypto API price increased)
                // Fall: end_price < initial_price (crypto API price decreased)
                // Neutral: end_price == initial_price (crypto API price stayed same)
                let actual_outcome =
                    self.calculate_outcome_from_prices(initial_price.price, end_price.price);

                // Check if prediction matches actual outcome
                let is_correct = prediction.outcome == actual_outcome;

                // Update prediction
                prediction.resolved = true;
                prediction.correct = Some(is_correct);

                // Update period data
                period_data.outcome = Some(actual_outcome);
                period_data.resolved = true;
                self.state.period_prices.insert(&period_key, period_data)?;

                // Update prediction in storage
                let prediction_key = format!(
                    "{:?}_{:?}_{}",
                    prediction.player_id,
                    period,
                    period_start.micros()
                );
                self.state
                    .predictions
                    .insert(&prediction_key, prediction.clone())?;

                // Award or penalize based on prediction correctness
                if is_correct {
                    // Correct prediction: award points based on period (Daily: 100, Weekly: 500, Monthly: 1000)
                    self.award_prediction_reward(&prediction.player_id, period)
                        .await?;
                    // If player is in a guild, award all guild members the same amount
                    self.award_guild_prediction_reward(&prediction.player_id, period)
                        .await?;
                } else {
                    // Wrong prediction: deduct points based on period (Daily: 100, Weekly: 500, Monthly: 1000)
                    self.penalize_prediction_loss(&prediction.player_id, period)
                        .await?;
                    // If player is in a guild, deduct the same amount from all guild members
                    self.penalize_guild_prediction_loss(&prediction.player_id, period)
                        .await?;
                }

                // Send resolution message
                self.runtime
                    .prepare_message(Message::PredictionResolved {
                        player_id: prediction.player_id,
                        period,
                        correct: is_correct,
                    })
                    .send_to(self.runtime.chain_id());
            }
        }

        Ok(())
    }

    /// Calculate the outcome (Rise, Fall, Neutral) based on price change from crypto API
    /// Compares end_price (from crypto API) to initial_price (from crypto API):
    /// Both prices come from crypto API providers like CoinMarketCap, CoinGecko, etc.
    /// - Rise: end_price > initial_price (crypto API price increased from initial)
    /// - Fall: end_price < initial_price (crypto API price decreased from initial)
    /// - Neutral: end_price == initial_price (crypto API price stayed the same as initial)
    fn calculate_outcome_from_prices(
        &self,
        initial_price: Amount,
        end_price: Amount,
    ) -> PriceOutcome {
        // Compare crypto API end_price to crypto API initial_price
        match end_price.cmp(&initial_price) {
            std::cmp::Ordering::Greater => PriceOutcome::Rise, // end_price > initial_price
            std::cmp::Ordering::Less => PriceOutcome::Fall,    // end_price < initial_price
            std::cmp::Ordering::Equal => PriceOutcome::Neutral, // end_price == initial_price
        }
    }

    /// Resolve all expired predictions when market price is updated
    async fn resolve_expired_predictions(
        &mut self,
        current_time: Timestamp,
    ) -> Result<(), ContractError> {
        // Check daily, weekly, and monthly periods
        let periods = vec![
            PredictionPeriod::Daily,
            PredictionPeriod::Weekly,
            PredictionPeriod::Monthly,
        ];

        for period in periods {
            let period_start = match period {
                PredictionPeriod::Daily => self.get_daily_period_start(current_time),
                PredictionPeriod::Weekly => self.get_weekly_period_start(current_time),
                PredictionPeriod::Monthly => self.get_monthly_period_start(current_time),
            };

            let period_key = format!("{:?}_{}", period, period_start.micros());

            if let Some(mut period_data) = self
                .state
                .period_prices
                .get(&period_key)
                .await?
                .map(|p| p.clone())
            {
                // Check if period has ended
                if current_time.micros() >= period_data.period_end.micros() {
                    // If initial price (start_price) is not set, set it from current crypto API price
                    // This shouldn't normally happen, but handle it as a fallback
                    if period_data.start_price.is_none() {
                        let current_price = self.state.current_market_price.get(); // From crypto API
                        period_data.start_price = Some(current_price.clone());
                        self.state
                            .period_prices
                            .insert(&period_key, period_data.clone())?;
                    }
                    // If end price is not set, set it from current crypto API price when period ends
                    // The oracle/admin should have called update_market_price() with the latest crypto API price
                    if period_data.end_price.is_none() {
                        let current_price = self.state.current_market_price.get(); // From crypto API (CoinMarketCap, CoinGecko, etc.)
                        period_data.end_price = Some(current_price.clone());
                        self.state
                            .period_prices
                            .insert(&period_key, period_data.clone())?;
                    }

                    // Now resolve all predictions for this period
                    // Iterate through all predictions and resolve those matching this period
                    let period_start_micros = period_start.micros();
                    let period_str = format!("{:?}", period);
                    let period_suffix = format!("_{}_{}", period_str, period_start_micros);

                    // Collect all prediction keys for this period
                    let mut prediction_keys_to_resolve = Vec::new();
                    self.state
                        .predictions
                        .for_each_index_value(|pred_key, prediction| {
                            // Check if this prediction matches the current period
                            // Prediction keys are in format: "{:?}_{:?}_{}", so we check the suffix
                            if pred_key.ends_with(&period_suffix) {
                                // Check if prediction is not already resolved
                                let pred = prediction.clone();
                                if !pred.resolved {
                                    prediction_keys_to_resolve.push(pred_key.clone());
                                }
                            }
                            Ok(())
                        })
                        .await
                        .expect("Failed to iterate predictions");

                    // Resolve each prediction for this period
                    for pred_key in prediction_keys_to_resolve {
                        // Get the prediction from storage
                        if let Some(mut prediction) = self
                            .state
                            .predictions
                            .get(&pred_key)
                            .await?
                            .map(|p| p.clone())
                        {
                            if !prediction.resolved {
                                // Try to resolve this prediction
                                let _ = self
                                    .resolve_prediction(&mut prediction, period, period_start)
                                    .await;
                                // Note: If resolution fails (e.g., prices not ready), it will be resolved on-demand later
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Award rewards to a player for a correct prediction
    /// Player gets points based on period: Daily: 100, Weekly: 500, Monthly: 1000
    async fn award_prediction_reward(
        &mut self,
        player_id: &PlayerId,
        period: PredictionPeriod,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(player_id).await?;

        // Award points based on period
        let reward = match period {
            PredictionPeriod::Daily => Amount::from_tokens(100), // Daily: 100 points
            PredictionPeriod::Weekly => Amount::from_tokens(500), // Weekly: 500 points
            PredictionPeriod::Monthly => Amount::from_tokens(1000), // Monthly: 1000 points
        };

        // Award tokens
        player.token_balance = player.token_balance.saturating_add(reward);
        player.total_earned = player.total_earned.saturating_add(reward);

        // Award XP (keep XP system for progression)
        let xp_reward = match period {
            PredictionPeriod::Daily => 50,
            PredictionPeriod::Weekly => 250,
            PredictionPeriod::Monthly => 500,
        };
        self.add_experience(&mut player, xp_reward).await?;
        self.state.players.insert(player_id, player.clone())?;

        // Update total supply
        let current_supply = self.state.total_supply.get();
        self.state
            .total_supply
            .set(current_supply.saturating_add(reward));

        // Update leaderboard after awarding points
        self.update_enhanced_leaderboard().await;

        // Broadcast player update to all chains for horizontal scaling
        self.broadcast_global_player_updated(
            *player_id,
            player.total_earned,
            player.total_profit,
            player.level,
        )
        .await;

        Ok(())
    }

    /// Penalize a player for a wrong prediction
    /// Player loses points based on period: Daily: 100, Weekly: 500, Monthly: 1000
    async fn penalize_prediction_loss(
        &mut self,
        player_id: &PlayerId,
        period: PredictionPeriod,
    ) -> Result<(), ContractError> {
        let mut player = self.get_player(player_id).await?;

        // Deduct points based on period
        let penalty = match period {
            PredictionPeriod::Daily => Amount::from_tokens(100), // Daily: 100 points
            PredictionPeriod::Weekly => Amount::from_tokens(500), // Weekly: 500 points
            PredictionPeriod::Monthly => Amount::from_tokens(1000), // Monthly: 1000 points
        };

        // Deduct tokens (ensure balance doesn't go negative)
        if player.token_balance >= penalty {
            player.token_balance = player.token_balance.saturating_sub(penalty);
            player.total_spent = player.total_spent.saturating_add(penalty);
        } else {
            // If player doesn't have enough, set balance to zero
            let deducted = player.token_balance;
            player.token_balance = Amount::ZERO;
            player.total_spent = player.total_spent.saturating_add(deducted);
        }

        self.state.players.insert(player_id, player)?;

        // Update total supply (burn tokens)
        let current_supply = self.state.total_supply.get();
        let to_burn = penalty.min(*current_supply);
        self.state
            .total_supply
            .set(current_supply.saturating_sub(to_burn));

        Ok(())
    }

    /// Award rewards to all guild members when a guild member makes a correct prediction
    /// Every player in the guild gets points based on period: Daily: 100, Weekly: 500, Monthly: 1000
    async fn award_guild_prediction_reward(
        &mut self,
        player_id: &PlayerId,
        period: PredictionPeriod,
    ) -> Result<(), ContractError> {
        let player = self.get_player(player_id).await?;

        // Check if player is in a guild
        if let Some(guild_id) = player.guild_id {
            if let Some(guild) = self.state.guilds.get(&guild_id).await?.map(|g| g.clone()) {
                // Award points based on period
                let reward = match period {
                    PredictionPeriod::Daily => Amount::from_tokens(100), // Daily: 100 points
                    PredictionPeriod::Weekly => Amount::from_tokens(500), // Weekly: 500 points
                    PredictionPeriod::Monthly => Amount::from_tokens(1000), // Monthly: 1000 points
                };

                // Award points to each guild member
                for member_id in &guild.members {
                    let mut member = self.get_player(member_id).await?;

                    member.token_balance = member.token_balance.saturating_add(reward);
                    member.total_earned = member.total_earned.saturating_add(reward);

                    // Award XP to guild members
                    let xp_reward = match period {
                        PredictionPeriod::Daily => 25,
                        PredictionPeriod::Weekly => 125,
                        PredictionPeriod::Monthly => 250,
                    };
                    self.add_experience(&mut member, xp_reward).await?;
                    self.state.players.insert(member_id, member)?;
                }

                // Update total supply
                let member_count = guild.members.len() as u128;
                let reward_tokens: u128 = reward.into();
                let total_reward_tokens = reward_tokens.saturating_mul(member_count);
                let total_reward = Amount::from_tokens(total_reward_tokens);
                let current_supply = self.state.total_supply.get();
                self.state
                    .total_supply
                    .set(current_supply.saturating_add(total_reward));

                // Update leaderboard after awarding guild points
                self.update_enhanced_leaderboard().await;
            }
        }

        Ok(())
    }

    /// Penalize all guild members when a guild member makes a wrong prediction
    /// Every player in the guild loses points based on period: Daily: 100, Weekly: 500, Monthly: 1000
    async fn penalize_guild_prediction_loss(
        &mut self,
        player_id: &PlayerId,
        period: PredictionPeriod,
    ) -> Result<(), ContractError> {
        let player = self.get_player(player_id).await?;

        // Check if player is in a guild
        if let Some(guild_id) = player.guild_id {
            if let Some(guild) = self.state.guilds.get(&guild_id).await?.map(|g| g.clone()) {
                // Deduct points based on period
                let penalty = match period {
                    PredictionPeriod::Daily => Amount::from_tokens(100), // Daily: 100 points
                    PredictionPeriod::Weekly => Amount::from_tokens(500), // Weekly: 500 points
                    PredictionPeriod::Monthly => Amount::from_tokens(1000), // Monthly: 1000 points
                };

                // Deduct points from each guild member
                for member_id in &guild.members {
                    let mut member = self.get_player(member_id).await?;

                    // Deduct tokens (ensure balance doesn't go negative)
                    if member.token_balance >= penalty {
                        member.token_balance = member.token_balance.saturating_sub(penalty);
                        member.total_spent = member.total_spent.saturating_add(penalty);
                    } else {
                        // If member doesn't have enough, set balance to zero
                        let deducted = member.token_balance;
                        member.token_balance = Amount::ZERO;
                        member.total_spent = member.total_spent.saturating_add(deducted);
                    }

                    self.state.players.insert(member_id, member)?;
                }

                // Update total supply (burn tokens)
                let member_count = guild.members.len() as u128;
                let penalty_tokens: u128 = penalty.into();
                let total_penalty_tokens = penalty_tokens.saturating_mul(member_count);
                let total_penalty = Amount::from_tokens(total_penalty_tokens);
                let current_supply = self.state.total_supply.get();
                let to_burn = total_penalty.min(*current_supply);
                self.state
                    .total_supply
                    .set(current_supply.saturating_sub(to_burn));
            }
        }

        Ok(())
    }

    // ============================================================================
    // Cross-Chain Broadcasting Functions (Horizontal Scaling)
    // ============================================================================

    /// Generate a unique message ID for deduplication
    /// Uses a combination of message type, content hash, chain_id, and timestamp
    fn generate_message_id(&mut self, message_type: &str, content: &str) -> String {
        let chain_id = self.runtime.chain_id();
        let timestamp = self.runtime.system_time();
        format!("{}:{}:{}:{}", message_type, chain_id, timestamp.micros(), content)
    }

    /// Check if a message has already been processed (idempotency check)
    async fn is_message_processed(&self, message_id: &str) -> bool {
        self.state
            .processed_message_ids
            .get(message_id)
            .await
            .ok()
            .flatten()
            .is_some()
    }

    /// Mark a message as processed
    async fn mark_message_processed(&mut self, message_id: &str) -> Result<(), ContractError> {
        let timestamp = self.runtime.system_time();
        self.state
            .processed_message_ids
            .insert(message_id, timestamp)?;
        Ok(())
    }

    /// Ensure this chain is registered in the chain registry for cross-chain coordination
    async fn ensure_chain_registered(&mut self) {
        let chain_id = self.runtime.chain_id();
        let current_time = self.runtime.system_time();

        // Check if already registered
        if self.state.subscribed_chains.get(&chain_id).await.ok().flatten().is_none() {
            // Register locally
            let _ = self.state.subscribed_chains.insert(&chain_id, current_time);

            // Broadcast registration to all other chains
            // This allows other chains to know about this chain
            self.broadcast_to_all_chains(Message::ChainRegistered {
                chain_id,
                timestamp: current_time,
            })
            .await;
        }
    }

    /// Broadcast a message to all chains that have the application
    /// This implements proper cross-chain messaging according to Linera's architecture
    async fn broadcast_to_all_chains(&mut self, message: Message) {
        let current_chain_id = self.runtime.chain_id();
        let mut chains_to_notify = Vec::new();

        // Collect all subscribed chains (excluding current chain to avoid self-messaging)
        self.state
            .subscribed_chains
            .for_each_index_value(|chain_id, _timestamp| {
                if chain_id != current_chain_id {
                    chains_to_notify.push(chain_id);
                }
                Ok(())
            })
            .await
            .ok();

        // Send message to each subscribed chain
        // Use with_authentication() and with_tracking() for reliable delivery
        for target_chain_id in chains_to_notify {
            self.runtime
                .prepare_message(message.clone())
                .with_authentication()
                .with_tracking()
                .send_to(target_chain_id);
        }
    }

    /// Broadcast player registration to all chains for horizontal scaling
    /// This allows all chains to maintain a global registry of players
    async fn broadcast_global_player_registered(
        &mut self,
        player_id: PlayerId,
        display_name: Option<String>,
    ) {
        let chain_id = self.runtime.chain_id();
        let current_time = self.runtime.system_time();
        
        // Generate unique message ID for deduplication
        let content = format!("{:?}:{:?}", player_id, display_name.as_deref().unwrap_or(""));
        let message_id = self.generate_message_id("GlobalPlayerRegistered", &content);
        
        // Update local global registry
        let display_name_clone = display_name.clone();
        let global_player = GlobalPlayerInfo {
            player_id,
            display_name: display_name_clone.clone(),
            chain_id,
            total_earned: Amount::ZERO,
            total_profit: Amount::ZERO,
            level: 1,
            last_updated: current_time,
        };
        let _ = self.state.global_players.insert(&player_id, global_player);

        // Broadcast to all subscribed chains (proper cross-chain messaging)
        self.broadcast_to_all_chains(Message::GlobalPlayerRegistered {
            player_id,
            display_name: display_name_clone,
            chain_id,
            message_id,
        })
        .await;
    }

    /// Broadcast market creation to all chains for horizontal scaling
    async fn broadcast_global_market_created(
        &mut self,
        market_id: MarketId,
        creator: PlayerId,
        title: String,
    ) {
        let chain_id = self.runtime.chain_id();
        let current_time = self.runtime.system_time();
        
        // Generate unique message ID for deduplication
        let content = format!("{}:{:?}", market_id, creator);
        let message_id = self.generate_message_id("GlobalMarketCreated", &content);
        
        // Update local global registry
        let global_market = GlobalMarketInfo {
            market_id,
            creator,
            title: title.clone(),
            chain_id,
            status: MarketStatus::Active,
            created_at: current_time,
        };
        let _ = self.state.global_markets.insert(&market_id, global_market);

        // Broadcast to all subscribed chains (proper cross-chain messaging)
        self.broadcast_to_all_chains(Message::GlobalMarketCreated {
            market_id,
            creator,
            title,
            chain_id,
            message_id,
        })
        .await;
    }

    /// Broadcast guild creation to all chains for horizontal scaling
    async fn broadcast_global_guild_created(
        &mut self,
        guild_id: GuildId,
        name: String,
        founder: PlayerId,
    ) {
        let chain_id = self.runtime.chain_id();
        let current_time = self.runtime.system_time();
        
        // Generate unique message ID for deduplication
        let content = format!("{}:{:?}", guild_id, founder);
        let message_id = self.generate_message_id("GlobalGuildCreated", &content);
        
        // Update local global registry
        let global_guild = GlobalGuildInfo {
            guild_id,
            name: name.clone(),
            founder,
            chain_id,
            member_count: 1,
            total_guild_profit: Amount::ZERO,
            created_at: current_time,
        };
        let _ = self.state.global_guilds.insert(&guild_id, global_guild);

        // Broadcast to all subscribed chains (proper cross-chain messaging)
        self.broadcast_to_all_chains(Message::GlobalGuildCreated {
            guild_id,
            name,
            founder,
            chain_id,
            message_id,
        })
        .await;
    }

    /// Broadcast player update to all chains when significant changes occur
    async fn broadcast_global_player_updated(
        &mut self,
        player_id: PlayerId,
        total_earned: Amount,
        total_profit: Amount,
        level: u32,
    ) {
        let chain_id = self.runtime.chain_id();
        let current_time = self.runtime.system_time();
        
        // Generate unique message ID for deduplication
        let content = format!("{:?}:{}:{}:{}", player_id, total_earned, total_profit, level);
        let message_id = self.generate_message_id("GlobalPlayerUpdated", &content);
        
        // Update local global registry
        if let Some(mut global_player) = self.state.global_players.get(&player_id).await.ok().flatten() {
            global_player.total_earned = total_earned;
            global_player.total_profit = total_profit;
            global_player.level = level;
            global_player.chain_id = chain_id;
            global_player.last_updated = current_time;
            let _ = self.state.global_players.insert(&player_id, global_player);
        }

        // Broadcast to all subscribed chains (proper cross-chain messaging)
        self.broadcast_to_all_chains(Message::GlobalPlayerUpdated {
            player_id,
            total_earned,
            total_profit,
            level,
            chain_id,
            timestamp: current_time, // Include timestamp for conflict resolution
            message_id,
        })
        .await;
    }

    /// Broadcast price update to all chains for horizontal scaling
    async fn broadcast_global_price_update(&mut self, price: Amount, timestamp: Timestamp) {
        let chain_id = self.runtime.chain_id();
        
        // Generate unique message ID for deduplication
        let content = format!("{}:{}", price, timestamp.micros());
        let message_id = self.generate_message_id("GlobalPriceUpdate", &content);
        
        // Broadcast to all subscribed chains (proper cross-chain messaging)
        self.broadcast_to_all_chains(Message::GlobalPriceUpdate {
            price,
            timestamp,
            chain_id,
            message_id,
        })
        .await;
    }

    /// Update global leaderboard by aggregating data from all chains
    async fn update_global_leaderboard(&mut self) {
        // Collect all players from global registry
        let mut all_players = Vec::new();
        self.state
            .global_players
            .for_each_index_value(|player_id, global_player| {
                all_players.push((player_id, global_player.clone()));
                Ok(())
            })
            .await
            .ok();

        // Sort by total profit and take top 50
        all_players.sort_by(|a, b| {
            b.1.total_profit
                .cmp(&a.1.total_profit)
                .then_with(|| b.1.level.cmp(&a.1.level))
        });

        let top_traders: Vec<LeaderboardEntry> = all_players
            .iter()
            .take(50)
            .map(|(player_id, global_player)| {
                // Calculate win rate (simplified - in real implementation would track wins/losses)
                let win_rate = if global_player.total_earned > Amount::ZERO {
                    // Estimate win rate based on profit vs earned ratio
                    let profit_tokens: u128 = global_player.total_profit.into();
                    let earned_tokens: u128 = global_player.total_earned.into();
                    if earned_tokens > 0 {
                        let profit_f64 = profit_tokens as f64;
                        let earned_f64 = earned_tokens as f64;
                        (profit_f64 / earned_f64).min(1.0).max(0.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };

                LeaderboardEntry {
                    player_id: *player_id,
                    display_name: global_player.display_name.clone(),
                    total_profit: global_player.total_profit,
                    win_rate,
                    level: global_player.level,
                }
            })
            .collect();

        // Update global leaderboard
        let mut global_leaderboard = self.state.global_leaderboard.get().clone();
        global_leaderboard.top_traders = top_traders;
        global_leaderboard.last_updated = self.runtime.system_time();
        self.state.global_leaderboard.set(global_leaderboard);
    }

    /// Update global leaderboard with a specific player's data
    async fn update_global_leaderboard_with_player(
        &mut self,
        player_id: PlayerId,
        total_profit: Amount,
        _win_rate: f64,
        level: u32,
    ) {
        // Update the player's global info
        if let Some(mut global_player) = self.state.global_players.get(&player_id).await.ok().flatten() {
            global_player.total_profit = total_profit;
            global_player.level = level;
            global_player.last_updated = self.runtime.system_time();
            let _ = self.state.global_players.insert(&player_id, global_player);
        }

        // Update global leaderboard
        self.update_global_leaderboard().await;
    }
}
