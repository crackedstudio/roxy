#![cfg_attr(target_arch = "wasm32", no_main)]

use linera_sdk::{
    linera_base_types::{Amount, Timestamp, WithContractAbi},
    views::View,
    Contract, ContractRuntime,
};
use predictive_manager::state::*;
use std::collections::BTreeMap;
use thiserror::Error;
use linera_sdk::views::ViewError;



// ============================================================================
// Errors
// ============================================================================

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("unauthorized")] Unauthorized,
    #[error("player already exists")] PlayerAlreadyExists,
    #[error("daily reward already claimed")] DailyRewardAlreadyClaimed,
    #[error("invalid outcome count")] InvalidOutcomeCount,
    #[error("duration too short")] DurationTooShort,
    #[error("insufficient balance")] InsufficientBalance,
    #[error("market not active")] MarketNotActive,
    #[error("market ended")] MarketEnded,
    #[error("invalid outcome")] InvalidOutcome,
    #[error("slippage exceeded")] SlippageExceeded,
    #[error("no position")] NoPosition,
    #[error("insufficient shares")] InsufficientShares,
    #[error("market not ready for voting")] MarketNotReadyForVoting,
    #[error("invalid resolution method")] InvalidResolutionMethod,
    #[error("already voted")] AlreadyVoted,
    #[error("market not ended")] MarketNotEnded,
    #[error("player not found")] PlayerNotFound,
    #[error("market not found")] MarketNotFound,
    #[error("guild not found")] GuildNotFound,
    #[error("already in guild")] AlreadyInGuild,
    #[error("not a guild member")] NotGuildMember,
    #[error("not admin")] NotAdmin,
    #[error("oracle not ready")] OracleNotReady,
    #[error("not resolved")] NotResolved,
    #[error("no winnings")] NoWinnings,
    #[error("insufficient level")] InsufficientLevel,
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

        match operation {
            predictive_manager::Operation::RegisterPlayer { display_name } => {
                let _ = self.register_player(player_id, display_name, current_time).await;
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
                fee_percent
            } => {
                let _ = self.create_market(
                    player_id,
                    title,
                    amount,
                    fee_percent,
                    current_time,
                ).await;
            }
            predictive_manager::Operation::BuyShares { 
                market_id,
                amount 
            } => {
                let _ = self.buy_shares(
                    player_id,
                    market_id,
                    amount,
                    current_time,
                ).await;
            }
            predictive_manager::Operation::SellShares { 
                market_id,
                amount 
            } => {
                let _ = self.sell_shares(
                    player_id,
                    market_id,
                    amount,
                    current_time,
                ).await;
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
        }
    }

    async fn execute_message(&mut self, message: Message) {
        match message {
            Message::MarketCreated { .. } => {}
            Message::MarketResolved { .. } => {}
            Message::TradeExecuted { .. } => {}
            Message::PlayerLeveledUp { .. } => {}
            Message::AchievementUnlocked { .. } => {}
            Message::GuildCreated { .. } => {}
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
            self.state.achievements.insert(&achievement.id.clone(), achievement)?;
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
        self.distribute_market_creator_fee(creator, creation_cost).await?;

        let market_id = self.generate_market_id().await?;
        
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
        self.state.players.insert(&creator, creator_player.clone())?;

        // Check for achievements after creating market
        self.check_achievements(&mut creator_player).await?;

        self
            .runtime
            .prepare_message(Message::MarketCreated { market_id, creator })
            .send_to(self.runtime.chain_id());

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
        let fee_tokens = base_payment_tokens.saturating_mul(market.fee_percent as u128).saturating_div(fee_divisor);
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
        let position = market
            .positions
            .entry(player_id)
            .or_insert(PlayerPosition {
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

        self
            .runtime
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
        let fee_tokens = amount_tokens.saturating_mul(market.fee_percent as u128).saturating_div(fee_divisor);
        let seller_fee = Amount::from_tokens(fee_tokens);
        let points_for_market = amount.saturating_sub(seller_fee);
        
        // Burn points from player (decrease their balance)
        player.token_balance = player.token_balance.saturating_sub(amount);
        player.total_spent = player.total_spent.saturating_add(amount);
        
        // Distribute trading fees: creator gets 98% of fee, platform gets 2% of fee
        self.distribute_trading_fees(market_id, seller_fee).await?;
        
        // Burn points from total supply (net after fee)
        let current_supply = self.state.total_supply.get();
        self.state.total_supply.set(current_supply.saturating_sub(points_for_market.min(*current_supply)));
        
        // Add liquidity to market (points available increase, minus fee)
        market.total_liquidity = market.total_liquidity.saturating_add(points_for_market);

        // Update position
        let position = market
            .positions
            .entry(player_id)
            .or_insert(PlayerPosition {
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

        self
            .runtime
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
        self.state.total_supply.set(current_supply.saturating_add(amount));
        
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

        self
            .runtime
            .prepare_message(Message::GuildCreated { guild_id: new_id, name })
            .send_to(self.runtime.chain_id());
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
    async fn join_guild(&mut self, player_id: PlayerId, guild_id: GuildId) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        if player.guild_id.is_some() {
            return Err(ContractError::AlreadyInGuild);
        }
        let mut guild = self.state.guilds.get(&guild_id).await?.ok_or(ContractError::GuildNotFound)?;
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
        let mut guild = self.state.guilds.get(&guild_id).await?.ok_or(ContractError::GuildNotFound)?;
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
    async fn contribute_to_guild(&mut self, player_id: PlayerId, amount: Amount) -> Result<(), ContractError> {
        let mut player = self.get_player(&player_id).await?;
        let guild_id = player.guild_id.ok_or(ContractError::NotGuildMember)?;
        if player.token_balance < amount { return Err(ContractError::InsufficientBalance); }
        
        // Deduct contribution from player's points (no external transfer needed)
        
        let mut guild = self.state.guilds.get(&guild_id).await?.ok_or(ContractError::GuildNotFound)?;
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
    async fn update_game_config(&mut self, caller: PlayerId, config: GameConfig) -> Result<(), ContractError> {
        let current = self.state.config.get();
        if let Some(admin) = current.admin {
            if caller != admin { return Err(ContractError::NotAdmin); }
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
    async fn update_enhanced_leaderboard(&mut self) {
        let mut top_traders = Vec::new();
        let mut top_guilds = Vec::new();
        
        // Collect all players and calculate enhanced scores
        let mut player_scores = Vec::new();
        self.state.players.for_each_index_value(|player_id, player| {
            let player = player.into_owned();
            // Use enhanced scoring (simplified to avoid self capture)
            let score: f64 = 100.0; // Simplified scoring for now
            player_scores.push((player_id, player, score));
            Ok(())
        }).await.expect("Failed to iterate players");
        
        // Sort by enhanced score (profit + win_rate + level + reputation)
        player_scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 50 traders
        for (player_id, player, _score) in player_scores.into_iter().take(50) {
            let win_rate = if player.markets_participated > 0 {
                (player.markets_won as f64 / player.markets_participated as f64) * 100.0
            } else {
                0.0
            };
            
            top_traders.push(LeaderboardEntry {
                player_id,
                display_name: player.display_name,
                total_profit: player.total_profit,
                win_rate,
                level: player.level,
            });
        }
        
        // Collect all guilds and calculate enhanced scores
        let mut guild_scores = Vec::new();
        self.state.guilds.for_each_index_value(|guild_id, guild| {
            let guild = guild.into_owned();
            // Simplified scoring to avoid self capture
            let score: f64 = 100.0; // Simplified scoring
            guild_scores.push((guild_id, guild, score));
            Ok(())
        }).await.expect("Failed to iterate guilds");
        
        // Sort by enhanced score (profit + member_count + level)
        guild_scores.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 20 guilds
        for (guild_id, guild, _score) in guild_scores.into_iter().take(20) {
            top_guilds.push(GuildLeaderboardEntry {
                guild_id,
                name: guild.name,
                total_profit: guild.total_guild_profit,
                member_count: guild.members.len() as u32,
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
        creation_fee: Amount
    ) -> Result<(), ContractError> {
        // Market creation fee (100 points) goes entirely to platform total supply
        let current_supply = self.state.total_supply.get();
        self.state.total_supply.set(current_supply.saturating_add(creation_fee));
        
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
        creator_fee_amount: Amount
    ) -> Result<(), ContractError> {
        let market = self.get_market(&market_id).await?;
        
        // Platform gets 2% of the creator's fee
        let platform_fee_percent = 2_u128; // 2% of creator's fee
        let fee_divisor = 100_u128;
        
        let creator_fee_tokens: u128 = creator_fee_amount.into();
        let platform_fee_tokens = creator_fee_tokens.saturating_mul(platform_fee_percent).saturating_div(fee_divisor);
        let platform_fee = Amount::from_tokens(platform_fee_tokens);
        let creator_keeps = creator_fee_amount.saturating_sub(platform_fee);
        
        // Give creator their share (creator's fee minus 2% platform fee)
        let mut creator_player = self.get_player(&market.creator).await?;
        creator_player.token_balance = creator_player.token_balance.saturating_add(creator_keeps);
        creator_player.total_earned = creator_player.total_earned.saturating_add(creator_keeps);
        self.state.players.insert(&market.creator, creator_player)?;
        
        // Add platform fee (2% of creator's fee) to total supply
        let current_supply = self.state.total_supply.get();
        self.state.total_supply.set(current_supply.saturating_add(platform_fee));
        
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
                    if self.check_achievement_requirement(player, &achievement.requirement).await? {
                        // Award achievement
                        player.achievements_earned.push(achievement_id);
                        player.token_balance = player.token_balance.saturating_add(achievement.reward_tokens);
                        player.total_earned = player.total_earned.saturating_add(achievement.reward_tokens);
                        player.experience_points += achievement.reward_xp;
                        
                        new_achievements.push(achievement_id);
                        
                        // Send achievement notification
                        self.runtime
                            .prepare_message(Message::AchievementUnlocked { 
                                player_id: player.id, 
                                achievement_id 
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
        requirement: &AchievementRequirement
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
            },
            AchievementRequirement::FirstBuy => {
                // Check if player has made at least one buy
                // Player has participated in markets (bought points from a market)
                Ok(player.markets_participated > 0)
            },
            AchievementRequirement::FirstSell => {
                // Check if player has sold points
                // A player has sold if they've spent points (sold points reduce balance)
                // We can also check if they've participated in markets as a proxy
                Ok(player.total_spent > Amount::ZERO)
            },
            AchievementRequirement::JoinGuild => Ok(player.guild_id.is_some()),
            AchievementRequirement::ReachLevel(level) => Ok(player.level >= *level),
            // Legacy requirements (kept for backward compatibility)
            AchievementRequirement::WinMarkets(count) => Ok(player.markets_won >= *count),
            AchievementRequirement::WinStreak(streak) => Ok(player.win_streak >= *streak),
            AchievementRequirement::TotalProfit(profit) => Ok(player.total_profit >= *profit),
            AchievementRequirement::ParticipateInMarkets(count) => Ok(player.markets_participated >= *count),
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
            },
        }
    }


    

}