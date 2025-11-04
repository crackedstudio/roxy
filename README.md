# Roxy Price - Blockchain-Based Prediction Market Game

![Roxy Logo](https://github.com/user-attachments/assets/f57fe362-4e7c-40d6-977d-cd521fec1452)

A decentralized prediction market game built on the Linera blockchain that combines social gaming, cryptocurrency price predictions, and a points-based economy.

##  Overview

Roxy Price is an innovative blockchain game where players predict cryptocurrency price movements (daily, weekly, and monthly), trade points in peer-to-peer markets, join guilds for collaborative gameplay, and compete on leaderboards. The game features a sophisticated progression system with levels, achievements, and experience points.

##  Key Features

###  Price Prediction System
- **Multi-Timeframe Predictions**: Make predictions on daily, weekly, and monthly cryptocurrency price movements
- **Oracle Integration**: Real-time price data from crypto APIs (CoinMarketCap, CoinGecko)
- **Outcome Types**: Predict Rise, Fall, or Neutral price movements
- **Reward System**: 
  - Daily predictions: 100 points per correct prediction
  - Weekly predictions: 500 points per correct prediction
  - Monthly predictions: 1000 points per correct prediction
- **Guild Synergy**: When a guild member makes a correct prediction, ALL guild members earn rewards

###  Point Trading Markets
- **Progressive Exchange Rate**: Level-based exchange system (10:1 ratio across all levels)
  - Level 1: Pay 10 points to receive 100 points
  - Level 2: Pay 100 points to receive 1000 points
  - Level N: Pay X/10 points to receive X points
- **Market Creation**: Level 5+ players with 10,000+ points can create markets
- **Custom Fee Structure**: Market creators set their own fee percentage (0-100%)
- **Fee Distribution**: 
  - Market creator keeps 98% of fees
  - Platform receives 2% of creator fees
- **Buying & Selling**: 
  - All players can buy points from markets
  - Only Level 5+ players can sell points to markets

###  Player Progression System
- **Experience & Leveling**: Exponential progression system
  - Level 1: 1,000 total XP required
  - Level 2: 4,000 total XP required (4x multiplier)
  - Level 3: 16,000 total XP required (4x multiplier)
  - Formula: `XP_required = 1000 × (4^(level-1))`
- **Reputation System**: Track player performance and trustworthiness
- **Win Streaks**: Monitor consecutive successful predictions
- **Achievement System**: Unlock rewards for milestones

###  Achievement System
Earn rewards for completing specific milestones:

| Achievement | Requirement | Reward | XP |
|------------|------------|--------|-----|
| Market Creator | Create first market | 100 points | 200 XP |
| First Buyer | Make first purchase | 50 points | 100 XP |
| First Seller | Make first sale | 50 points | 100 XP |
| Guild Member | Join a guild | 150 points | 300 XP |
| Level 2 Achiever | Reach level 2 | 200 points | 400 XP |
| Level 3 Achiever | Reach level 3 | 400 points | 800 XP |
| Level 5 Achiever | Reach level 5 | 1000 points | 2000 XP |

###  Guild System
- **Create Guilds**: Form social groups for collaborative gameplay
- **Shared Rewards**: Guild members share prediction rewards
- **Shared Penalties**: Guild members share prediction losses
- **Guild Pool**: Contribute points to collective fund
- **Guild Leaderboard**: Compete with other guilds based on total member earnings

###  Leaderboard System
- **Player Rankings**: Top 50 traders by total points earned
- **Guild Rankings**: Top 20 guilds by total member points
- **Statistics Tracked**:
  - Total profit (points earned)
  - Win rate percentage
  - Player level
  - Guild member count

###  Daily Login Rewards
- Claim free points every 24 hours
- Encourages regular player engagement
- Configurable reward amounts

##  Technical Stack

- **Blockchain**: Linera Protocol v0.15.4
- **Smart Contract Language**: Rust
- **State Management**: Linera Views (persistent storage)
- **Oracle Integration**: External price feed from crypto APIs

##  Getting Started

## 📖 Game Mechanics

### For New Players

1. **Register**: Create your player account
   - Receive initial token allocation (configurable)
   - Start at Level 1 with 0 XP

2. **Claim Daily Rewards**: 
   - Log in daily to claim free points
   - Build your balance for trading

3. **Make Predictions**:
   - Predict daily crypto price movements (100 point reward)
   - Predict weekly movements (500 point reward)
   - Predict monthly movements (1000 point reward)

4. **Buy Points from Markets**:
   - Browse available markets
   - Progressive exchange rate: pay 10% of desired amount
   - Market creator earns fees on your purchase

5. **Progress & Earn**:
   - Complete achievements
   - Level up for better trading capabilities
   - Join guilds for shared rewards

### For Advanced Players (Level 5+)

1. **Create Markets**:
   - Requires Level 5 and 10,000+ points
   - Pay 100 point creation fee
   - Set custom fee percentage (0-100%)
   - Earn trading fees from buyers and sellers

2. **Sell Points to Markets**:
   - Level 5+ only
   - Convert your points back to market liquidity
   - Pay market creator's fee

3. **Build Guild Empire**:
   - Create or join guilds
   - Share prediction rewards with members
   - Compete on guild leaderboard

##  Operations

### Player Operations
- `RegisterPlayer`: Create new player account
- `UpdateProfile`: Change display name
- `ClaimDailyReward`: Claim 24-hour login bonus
- `PredictDailyOutcome`: Make daily price prediction
- `PredictWeeklyOutcome`: Make weekly price prediction
- `PredictMonthlyOutcome`: Make monthly price prediction

### Market Operations
- `CreateMarket`: Create new point trading market (Level 5+)
- `BuyShares`: Purchase points from a market
- `SellShares`: Sell points to a market (Level 5+)

### Guild Operations
- `CreateGuild`: Form a new guild
- `JoinGuild`: Join existing guild
- `LeaveGuild`: Leave current guild
- `ContributeToGuild`: Add points to guild pool

### Admin Operations
- `UpdateGameConfig`: Modify game parameters
- `MintPoints`: Create additional point supply
- `UpdateMarketPrice`: Update crypto prices from oracle

##  Security Features

- **Authentication**: All operations require authenticated signer
- **Authorization**: Admin-only operations protected
- **Level Requirements**: Market creation and selling restricted by level
- **Balance Checks**: Prevent overdraft on all transactions
- **Fee Validation**: Market fees capped at 100%

##  Economic Model

### Point Supply
- Initial player allocation (configurable)
- Daily login rewards
- Prediction rewards (correct guesses)
- Achievement rewards
- Market creation fees (100 points → platform)
- Trading fees (2% → platform, 98% → market creator)

### Point Burn
- Wrong predictions (100-1000 points)
- Market creation cost (100 points)
- Trading transaction fees
- Selling points to markets

### Progressive Trading
The exchange rate scales with player level to reward progression:
- **Buying Formula**: `payment = desired_points / 10`
- **Example Level 1**: Want 100 points? Pay 10 points
- **Example Level 2**: Want 1000 points? Pay 100 points
- **Example Level 5**: Want 10000 points? Pay 1000 points

This creates natural market dynamics where higher-level players can make larger trades.

##  Leaderboard Ranking

### Player Ranking
Players are ranked by **total points earned** (`total_earned`), which includes:
- Initial allocation
- Daily rewards
- Prediction rewards
- Achievement rewards
- Trading profits

### Guild Ranking
Guilds are ranked by **sum of all member earnings**, encouraging guilds to recruit active traders and predictors.

##  Oracle System

The prediction system relies on external price data from cryptocurrency APIs:

1. **Admin/Oracle** calls `UpdateMarketPrice` with latest crypto price
2. Contract captures **initial price** at period start (from crypto API)
3. Contract captures **end price** at period end (from crypto API)
4. **Outcome Calculation**:
   - Rise: `end_price > initial_price`
   - Fall: `end_price < initial_price`
   - Neutral: `end_price == initial_price`
5. Players' predictions compared to actual outcome
6. Rewards/penalties distributed automatically



# Roxy Price - Technical Architecture Documentation

## Table of Contents
1. [System Overview](#system-overview)
2. [Contract Architecture](#contract-architecture)
3. [State Management](#state-management)
4. [Core Subsystems](#core-subsystems)
5. [Economic Model](#economic-model)
6. [Data Flow](#data-flow)
7. [Security Architecture](#security-architecture)
8. [Oracle Integration](#oracle-integration)
9. [Scalability Considerations](#scalability-considerations)

---

## 1. System Overview

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Roxy Price Platform                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Price       │  │   Market     │  │    Guild     │      │
│  │  Prediction  │  │   Trading    │  │    System    │      │
│  │  Engine      │  │   System     │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Player      │  │  Achievement │  │  Leaderboard │      │
│  │  Management  │  │  System      │  │  System      │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│              Linera Smart Contract Layer                     │
├─────────────────────────────────────────────────────────────┤
│                  Linera Blockchain                           │
└─────────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
    Players API          Oracle API           Admin API
```



## 2. Contract Architecture

### 2.1 Contract Structure

The contract follows the **Linera Contract Pattern**:
- **State**: Persistent storage using Linera Views
- **Runtime**: Access to blockchain context (time, chain ID, authentication)

### 2.2 Operation Flow

```
User Request
    ↓
Authentication Check (runtime.authenticated_signer())
    ↓
Operation Validation (balance, level, permissions)
    ↓
State Mutation (update MapView/RegisterView)
    ↓
Side Effects (messages, events, rewards)
    ↓
State Persistence (automatic via Linera SDK)
```

### 2.3 Message System

The contract uses Linera's message system for event broadcasting:

```rust
pub enum Message {
    MarketCreated { market_id: MarketId, creator: PlayerId },
    MarketResolved { market_id: MarketId, outcome_id: OutcomeId },
    TradeExecuted { player_id, market_id, outcome_id, shares, price },
    PlayerLeveledUp { player_id: PlayerId, new_level: u32 },
    AchievementUnlocked { player_id: PlayerId, achievement_id: u64 },
    GuildCreated { guild_id: GuildId, name: String },
    PredictionMade { player_id, period, outcome },
    PredictionResolved { player_id, period, correct: bool },
}
```

Messages are sent to the current chain for event tracking and frontend updates.

---

## 3. State Management

### 3.1 State Schema

```rust
pub struct PredictionMarketState {
    // Core Configuration
    pub config: RegisterView<GameConfig>,
    pub total_supply: RegisterView<Amount>,
    pub next_market_id: RegisterView<MarketId>,
    
    // Player Data
    pub players: MapView<PlayerId, Player>,
    
    // Market Data
    pub markets: MapView<MarketId, Market>,
    
    // Guild Data
    pub guilds: MapView<GuildId, Guild>,
    
    // Achievement System
    pub achievements: MapView<u64, Achievement>,
    
    // Leaderboard
    pub leaderboard: RegisterView<Leaderboard>,
    
    // Price Prediction System
    pub current_market_price: RegisterView<MarketPrice>,
    pub predictions: MapView<String, PlayerPrediction>,
    pub period_prices: MapView<String, PeriodPriceData>,
}
```

### 3.2 Data Models

#### Player Model
```rust
pub struct Player {
    id: PlayerId,
    display_name: Option<String>,
    registration_time: Timestamp,
    last_login: Timestamp,
    
    // Economics
    token_balance: Amount,
    total_earned: Amount,
    total_spent: Amount,
    
    // Progression
    level: u32,
    experience_points: u64,
    reputation: u32,
    
    // Statistics
    markets_participated: u32,
    markets_won: u32,
    total_profit: Amount,
    win_streak: u32,
    best_win_streak: u32,
    
    // Social
    guild_id: Option<GuildId>,
    achievements_earned: Vec<u64>,
    active_markets: Vec<MarketId>,
}
```

#### Market Model
```rust
pub struct Market {
    id: MarketId,
    creator: PlayerId,
    title: String,
    amount: Amount,                    // Initial liquidity
    fee_percent: u8,                   // Creator's fee (0-100)
    creation_time: Timestamp,
    status: MarketStatus,
    total_liquidity: Amount,           // Current liquidity
    positions: BTreeMap<PlayerId, PlayerPosition>,
    total_participants: u32,
}
```

#### Prediction Model
```rust
pub struct PlayerPrediction {
    player_id: PlayerId,
    period: PredictionPeriod,          // Daily/Weekly/Monthly
    outcome: PriceOutcome,             // Rise/Fall/Neutral
    prediction_time: Timestamp,
    period_start: Timestamp,
    resolved: bool,
    correct: Option<bool>,
}

pub struct PeriodPriceData {
    period_start: Timestamp,
    period_end: Timestamp,
    start_price: Option<MarketPrice>,  // From crypto API
    end_price: Option<MarketPrice>,    // From crypto API
    outcome: Option<PriceOutcome>,
    resolved: bool,
}
```

### 3.3 Storage Patterns

**MapView Usage**: For indexed collections with unique keys
```rust
// Player lookup by ID
self.state.players.get(&player_id).await?

// Market lookup by ID
self.state.markets.get(&market_id).await?

// Prediction lookup by composite key
let key = format!("{:?}_{:?}_{}", player_id, period, period_start);
self.state.predictions.get(&key).await?
```

**RegisterView Usage**: For single global values
```rust
// Game configuration
self.state.config.get()

// Total point supply
self.state.total_supply.get()

// Current market price
self.state.current_market_price.get()
```

---

## 4. Core Subsystems

### 4.1 Player Management Subsystem

**Responsibilities**:
- Player registration and profile management
- Experience and leveling system
- Daily reward distribution
- Balance management

**Key Functions**:
```rust
async fn register_player(player_id, display_name, current_time)
async fn update_player_profile(player_id, display_name)
async fn claim_daily_reward(player_id, current_time)
async fn add_experience(player: &mut Player, xp: u64)
```

**Leveling Algorithm**:
```
Level 1: 1,000 total XP
Level 2: 4,000 total XP (4x multiplier)
Level 3: 16,000 total XP (4x multiplier)
Level N: 1,000 × 4^(N-1) total XP

Formula: XP_required(N) = 1000 × 4^(N-1)
```

This exponential progression encourages continuous engagement while becoming progressively challenging.

### 4.2 Market Trading Subsystem

**Responsibilities**:
- Market creation and lifecycle management
- Point buying and selling
- Fee calculation and distribution
- Liquidity management

**Market Creation Flow**:
```
1. Validate: Level ≥ 5, Balance ≥ 10,000 points
2. Deduct: 100 point creation fee
3. Create: New market with custom fee percentage
4. Distribute: Creation fee to platform (total_supply)
5. Emit: MarketCreated message
```

**Trading Flow (Buy)**:
```
1. Validate: Market active, sufficient buyer balance
2. Calculate: 
   - Base payment = desired_points / 10
   - Fee = base_payment × market.fee_percent / 100
   - Total payment = base_payment + fee
3. Transfer:
   - Buyer pays: total_payment
   - Buyer receives: desired_points (from market liquidity)
   - Creator receives: base_payment (98% after platform cut)
   - Platform receives: 2% of fee
4. Update: Market liquidity, player positions
5. Emit: TradeExecuted message
```

**Progressive Exchange Rate**:
```rust
// Level-agnostic formula (works for all levels 1 to ∞)
let payment_tokens = desired_points_tokens / 10;

Examples:
- Want 100 points → pay 10 points (10:1 ratio)
- Want 1,000 points → pay 100 points (10:1 ratio)
- Want 10,000 points → pay 1,000 points (10:1 ratio)
```

This creates a consistent 10% cost regardless of level, maintaining economic balance.

### 4.3 Price Prediction Subsystem

**Architecture**:
```
┌──────────────────────────────────────────────────────┐
│                 Oracle/Admin                          │
│          (Fetches from Crypto APIs)                   │
└───────────────────┬──────────────────────────────────┘
                    │ update_market_price(price)
                    ▼
┌──────────────────────────────────────────────────────┐
│           Smart Contract Storage                      │
│  - current_market_price (latest from API)            │
│  - period_prices (start_price, end_price)            │
│  - predictions (player predictions)                   │
└───────────────────┬──────────────────────────────────┘
                    │ resolve_expired_predictions()
                    ▼
┌──────────────────────────────────────────────────────┐
│            Prediction Resolution                      │
│  1. Compare end_price vs start_price (both from API) │
│  2. Determine outcome: Rise/Fall/Neutral              │
│  3. Check player predictions                          │
│  4. Award/penalize players and guilds                 │
└──────────────────────────────────────────────────────┘
```

**Prediction Flow**:
```
1. Player submits prediction (Rise/Fall/Neutral)
2. Contract captures initial_price from current_market_price
3. Period elapses (24h for daily, 7d for weekly, 30d for monthly)
4. Oracle updates current_market_price with latest crypto API price
5. Contract captures end_price from current_market_price
6. Contract calculates outcome: compare end_price vs initial_price
7. Contract resolves player prediction (correct/incorrect)
8. Contract distributes rewards or penalties
```

**Outcome Calculation**:
```rust
fn calculate_outcome_from_prices(initial_price, end_price) -> PriceOutcome {
    match end_price.cmp(&initial_price) {
        Greater => PriceOutcome::Rise,    // Price increased
        Less => PriceOutcome::Fall,       // Price decreased
        Equal => PriceOutcome::Neutral,   // Price unchanged
    }
}
```

**Reward/Penalty Schedule**:
| Period | Correct Reward | Wrong Penalty | XP Reward |
|--------|----------------|---------------|-----------|
| Daily | 100 points | -100 points | 50 XP |
| Weekly | 500 points | -500 points | 250 XP |
| Monthly | 1000 points | -1000 points | 500 XP |

**Guild Multiplier**:
When a guild member makes a prediction:
- **Correct**: ALL guild members receive the reward
- **Wrong**: ALL guild members lose the penalty amount

This creates strong incentives for guild coordination and collective decision-making.

### 4.4 Guild System

**Guild Model**:
```rust
pub struct Guild {
    id: GuildId,
    name: String,
    founder: PlayerId,
    members: Vec<PlayerId>,
    creation_time: Timestamp,
    total_guild_profit: Amount,
    guild_level: u32,
    shared_pool: Amount,
}
```

**Guild Operations**:
- **Create**: Founder creates guild (must not be in another guild)
- **Join**: Players join existing guilds
- **Leave**: Members can leave guilds
- **Contribute**: Members add points to shared pool
- **Collective Rewards**: All members earn from member predictions

**Guild Ranking Algorithm**:
```rust
// Sum all member earnings
guild_total_points = sum(member.total_earned for each member)

// Sort guilds by total points (descending)
// Top 20 displayed on leaderboard
```

### 4.5 Achievement System

**Achievement Triggers**:
```rust
pub enum AchievementRequirement {
    CreateMarket,                    // Create first market
    FirstBuy,                        // Make first purchase
    FirstSell,                       // Make first sale
    JoinGuild,                       // Join a guild
    ReachLevel(u32),                // Reach specific level
    WinMarkets(u32),                // Win N markets
    WinStreak(u32),                 // Achieve N win streak
    TotalProfit(Amount),            // Earn total profit
    ParticipateInMarkets(u32),      // Participate in N markets
    CreateMarkets(u32),             // Create N markets
}
```

**Achievement Check Flow**:
```
1. Player completes action (e.g., creates market)
2. Contract calls check_achievements(&mut player)
3. For each achievement:
   - Check if player already earned it
   - Check if requirement met
   - If yes: award tokens + XP, add to player.achievements_earned
4. Emit AchievementUnlocked message
5. Update player state
```

**Rewards Distribution**:
```rust
Achievement::award() {
    player.token_balance += achievement.reward_tokens;
    player.total_earned += achievement.reward_tokens;
    player.experience_points += achievement.reward_xp;
    total_supply += achievement.reward_tokens;
}
```

### 4.6 Leaderboard System

**Ranking Algorithms**:

**Player Ranking**:
```rust
// Rank by total_earned (all-time earnings)
player_score = player.total_earned

// Calculate win rate for display
win_rate = (markets_won / markets_participated) × 100

// Sort descending, take top 50
```

**Guild Ranking**:
```rust
// Sum all member total_earned
guild_score = sum(member.total_earned for each guild.member)

// Sort descending, take top 20
```

**Update Triggers**:
- Market creation fee distribution
- Trading fee distribution
- Prediction reward distribution

**Leaderboard Model**:
```rust
pub struct Leaderboard {
    top_traders: Vec<LeaderboardEntry>,     // Top 50 players
    top_guilds: Vec<GuildLeaderboardEntry>,  // Top 20 guilds
    last_updated: Timestamp,
}
```

---

## 5. Economic Model

### 5.1 Point Supply Mechanics

**Sources (Minting)**:
- Initial player allocation (configurable, default: varies)
- Daily login rewards
- Correct predictions (100/500/1000 points)
- Guild prediction rewards (multiplied by member count)
- Achievement rewards (50-1000 points)
- Market creation fees (100 points → platform)
- Trading platform fees (2% of creator fee)

**Sinks (Burning)**:
- Wrong predictions (100/500/1000 points)
- Guild prediction penalties (multiplied by member count)
- Market trading (net burn after fees)

**Total Supply Tracking**:
```rust
self.state.total_supply.get()  // Current circulating supply
self.state.total_supply.set(new_supply)  // Update supply
```

### 5.2 Fee Structure

**Market Creation**:
```
Cost: 100 points (flat fee)
Distribution: 100% to platform (total_supply)
```

**Trading Fees**:
```
Market Creator Fee: X% (set by creator, 0-100%)
Platform Cut: 2% of creator fee
Creator Keeps: 98% of creator fee

Example with 10% creator fee on 1000 point trade:
- Buyer pays: 1000 + 100 (fee) = 1100 points
- Creator fee: 100 points
- Platform gets: 2 points (2% of 100)
- Creator keeps: 98 points (98% of 100)
```

**Buy Formula**:
```rust
base_payment = desired_points / 10
creator_fee = base_payment × market.fee_percent / 100
total_payment = base_payment + creator_fee

platform_cut = creator_fee × 0.02
creator_keeps = creator_fee × 0.98
```

**Sell Formula**:
```rust
creator_fee = amount × market.fee_percent / 100
points_to_market = amount - creator_fee

platform_cut = creator_fee × 0.02
creator_keeps = creator_fee × 0.98
```

### 5.3 Balance Safety

**Overflow Protection**:
```rust
// All arithmetic uses saturating operations
player.token_balance = player.token_balance.saturating_add(reward);
player.token_balance = player.token_balance.saturating_sub(cost);
```

**Underflow Protection**:
```rust
// Check balance before deducting
if player.token_balance < cost {
    return Err(ContractError::InsufficientBalance);
}
```

**Zero-Floor Penalties**:
```rust
// For prediction penalties, set balance to zero if insufficient
if player.token_balance >= penalty {
    player.token_balance = player.token_balance.saturating_sub(penalty);
} else {
    player.token_balance = Amount::ZERO;
}
```

---

## 6. Data Flow

### 6.1 Player Registration Flow

```
User → RegisterPlayer{display_name}
  ↓
Check: Player doesn't exist
  ↓
Create: Player{initial_tokens, level: 1, xp: 0}
  ↓
Update: total_supply += initial_tokens
  ↓
Store: players.insert(player_id, player)
  ↓
Response: Success
```

### 6.2 Market Creation Flow

```
User → CreateMarket{title, amount, fee_percent}
  ↓
Validate: level ≥ 5, balance ≥ 10,000, balance ≥ 100 (fee)
  ↓
Deduct: player.balance -= 100 (creation fee)
  ↓
Create: Market{creator, title, amount, fee_percent, status: Active}
  ↓
Distribute: total_supply += 100 (platform fee)
  ↓
Update: player.active_markets.push(market_id)
  ↓
Check: Achievements (Market Creator)
  ↓
Emit: MarketCreated message
  ↓
Response: Success
```

### 6.3 Buy Points Flow

```
User → BuyShares{market_id, amount}
  ↓
Validate: Market active, market has liquidity
  ↓
Calculate:
  - points_to_receive = min(amount, market.liquidity)
  - base_payment = points_to_receive / 10
  - creator_fee = base_payment × market.fee_percent / 100
  - total_payment = base_payment + creator_fee
  ↓
Validate: player.balance ≥ total_payment
  ↓
Transfer:
  - player.balance -= total_payment
  - player.balance += points_to_receive (from market)
  - market.liquidity -= points_to_receive
  - creator.balance += base_payment
  - Distribute creator_fee (98% creator, 2% platform)
  ↓
Update: Position, XP (+10), statistics
  ↓
Check: Achievements (First Buyer)
  ↓
Emit: TradeExecuted message
  ↓
Response: Success
```

### 6.4 Prediction Flow

```
User → PredictDailyOutcome{outcome}
  ↓
Calculate: period_start (current day start)
  ↓
Validate: No existing prediction for this period
  ↓
Create: PlayerPrediction{player_id, period, outcome, period_start}
  ↓
Initialize: PeriodPriceData{start_price: current_market_price}
  ↓
Store: predictions.insert(key, prediction)
  ↓
Emit: PredictionMade message
  ↓
[Wait for period to end]
  ↓
Oracle → UpdateMarketPrice{price} (at period end)
  ↓
Capture: period_data.end_price = current_market_price
  ↓
Calculate: outcome = compare(end_price, start_price)
  ↓
Resolve: prediction.correct = (prediction.outcome == outcome)
  ↓
If Correct:
  - Player: +100/500/1000 points, +XP
  - Guild: All members +100/500/1000 points, +XP
  - Update: total_supply
If Wrong:
  - Player: -100/500/1000 points
  - Guild: All members -100/500/1000 points
  - Burn: from total_supply
  ↓
Emit: PredictionResolved message
  ↓
Update: Leaderboard
```

### 6.5 Oracle Price Update Flow

```
Oracle/Admin → UpdateMarketPrice{price}
  ↓
Validate: Caller is admin
  ↓
Update: current_market_price = {price, timestamp}
  ↓
Resolve Expired Predictions:
  ↓
  For each period (Daily/Weekly/Monthly):
    ↓
    Check: current_time ≥ period_end
    ↓
    If period ended:
      - Set: period_data.end_price = current_market_price
      - Calculate: outcome = compare(end_price, start_price)
      - Resolve: All predictions for this period
      - Award/Penalize: Players and guilds
      - Emit: PredictionResolved messages
  ↓
Response: Success
```

---

## 7. Security Architecture

### 7.1 Authentication

**Signer-Based Auth**:
```rust
let player_id = self.runtime.authenticated_signer().unwrap();
```

Every operation requires an authenticated signer. The blockchain ensures the signer owns the private key for the player_id.

### 7.2 Authorization

**Admin-Only Operations**:
```rust
let config = self.state.config.get();
if let Some(admin) = config.admin {
    if caller != admin {
        return Err(ContractError::NotAdmin);
    }
}
```

Admin operations:
- `UpdateGameConfig`: Modify game parameters
- `MintPoints`: Create point supply
- `UpdateMarketPrice`: Oracle price updates

**Level-Based Restrictions**:
```rust
// Market creation requires Level 5
if player.level < 5 {
    return Err(ContractError::InsufficientLevel);
}

// Selling requires Level 5
if player.level < 5 {
    return Err(ContractError::InsufficientLevel);
}
```

**Balance Checks**:
```rust
// Before any deduction
if player.token_balance < cost {
    return Err(ContractError::InsufficientBalance);
}
```

### 7.3 Input Validation

**Fee Validation**:
```rust
// Market creator fee capped at 100%
if fee_percent > 100 {
    return Err(ContractError::InvalidOutcome);
}
```

**Cooldown Validation**:
```rust
// Daily reward: 24-hour cooldown
let time_diff = current_time.micros() - player.last_login.micros();
let one_day_micros = 24 * 60 * 60 * 1_000_000;
if time_diff < one_day_micros {
    return Err(ContractError::DailyRewardAlreadyClaimed);
}
```

**Duplicate Prevention**:
```rust
// Prevent duplicate predictions for same period
let prediction_key = format!("{:?}_{:?}_{}", player_id, period, period_start);
if self.state.predictions.contains_key(&prediction_key).await? {
    return Err(ContractError::InvalidOutcome);
}
```

### 7.4 State Consistency

**Atomic Updates**:
All state changes within a single operation are atomic. If any step fails, the entire transaction reverts.

**View Isolation**:
Linera Views ensure concurrent operations don't interfere:
```rust
// Each MapView key is independently locked
self.state.players.insert(&player_id, player)?;
self.state.markets.insert(&market_id, market)?;
```

**Error Handling**:
```rust
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("unauthorized")] Unauthorized,
    #[error("player already exists")] PlayerAlreadyExists,
    #[error("insufficient balance")] InsufficientBalance,
    // ... comprehensive error types
}
```

---

## 8. Oracle Integration

### 8.1 Oracle Architecture

```
┌──────────────────────────────────────────────────────┐
│              External Crypto Price APIs               │
│      (CoinMarketCap, CoinGecko, Binance, etc.)       │
└───────────────────┬──────────────────────────────────┘
                    │ HTTPS API Calls
                    ▼
┌──────────────────────────────────────────────────────┐
│                Oracle Service                         │
│  - Fetches latest crypto prices                      │
│  - Converts to Amount type                            │
│  - Calls smart contract                               │
└───────────────────┬──────────────────────────────────┘
                    │ update_market_price(price)
                    ▼
┌──────────────────────────────────────────────────────┐
│           Roxy Price Smart Contract                   │
│  - Stores price in current_market_price              │
│  - Updates period_prices (start/end)                 │
│  - Resolves expired predictions                       │
│  - Distributes rewards/penalties                      │
└──────────────────────────────────────────────────────┘
```

### 8.2 Price Update Protocol

**Update Function**:
```rust
async fn update_market_price(
    &mut self,
    caller: PlayerId,      // Must be admin
    price: Amount,         // Price from crypto API
    current_time: Timestamp,
) -> Result<(), ContractError>
```

**Price Storage**:
```rust
pub struct MarketPrice {
    price: Amount,         // Latest price from API
    timestamp: Timestamp,  // When price was captured
}
```

**Period Tracking**:
```rust
pub struct PeriodPriceData {
    period_start: Timestamp,
    period_end: Timestamp,
    start_price: Option<MarketPrice>,  // Captured at period start
    end_price: Option<MarketPrice>,    // Captured at period end
    outcome: Option<PriceOutcome>,     // Rise/Fall/Neutral
    resolved: bool,
}
```

### 8.3 Oracle Implementation Example

**Off-Chain Oracle Service** (pseudo-code):
```python
# Oracle service runs continuously
while True:
    # Fetch latest BTC price from CoinGecko
    response = requests.get('https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd')
    btc_price = response.json()['bitcoin']['usd']
    
    # Convert to Amount (assuming 1 point = $1 USD)
    price_amount = Amount.from_tokens(btc_price)
    
    # Call smart contract as admin
    contract.update_market_price(
        caller=admin_account,
        price=price_amount,
        current_time=now()
    )
    
    # Wait before next update
    time.sleep(60)  # Update every minute
```

### 8.4 Prediction Resolution Triggers

**Automatic Resolution**:
When `update_market_price` is called, the contract automatically:

1. **Checks Expired Periods**:
```rust
for period in [Daily, Weekly, Monthly] {
    if current_time >= period_end {
        // Period has ended
    }
}
```

2. **Captures End Price**:
```rust
if period_data.end_price.is_none() {
    period_data.end_price = Some(current_market_price.clone());
}
```

3. **Resolves Predictions**:
```rust
for each prediction in period {
    outcome = calculate_outcome(start_price, end_price);
    is_correct = (prediction.outcome == outcome);
    award_or_penalize(player, is_correct);
}
```

**On-Demand Resolution**:
When players query their prediction results:
```rust
async fn get_daily_outcome(&mut self, player_id: PlayerId) -> Result<bool, ContractError> {
    let prediction = self.get_prediction(player_id, Daily)?;
    if !prediction.resolved {
        self.resolve_prediction(&mut prediction, Daily, period_start).await?;
    }
    Ok(prediction.correct.unwrap_or(false))
}
```

---

## 9. Scalability Considerations

### 9.1 Storage Optimization

**Key Strategies**:
- Use composite keys for predictions: `"{player_id}_{period}_{period_start}"`
- Limit leaderboard to top 50 players and top 20 guilds
- Period prices stored per period (not per player)
- Achievements stored globally (not duplicated per player)

**Storage Growth**:
```
Players: O(n) where n = number of players
Markets: O(m) where m = number of markets
Predictions: O(n × p × t) where:
  - n = number of players
  - p = prediction periods (3: daily/weekly/monthly)
  - t = time periods (grows continuously)
Guilds: O(g) where g = number of guilds
Achievements: O(7) fixed (7 predefined achievements)
```

**Mitigation Strategies**:
- Implement prediction archival (move old predictions to cold storage)
- Set max active markets per player (currently unlimited)
- Periodic leaderboard snapshots (reduce recalculation frequency)
- Lazy resolution (resolve on-demand vs. batch resolution)

### 9.2 Computational Complexity

**Operation Complexities**:

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| RegisterPlayer | O(1) | Single map insertion |
| CreateMarket | O(1) | Single map insertion + checks |
| BuyShares | O(1) | Direct map access |
| SellShares | O(1) | Direct map access |
| PredictOutcome | O(1) | Single map insertion |
| ResolveExpiredPredictions | O(n × p) | n players, p periods |
| UpdateLeaderboard | O(n log n) | Sort all players/guilds |
| CheckAchievements | O(a) | a = number of achievements (fixed 7) |

**Performance Bottlenecks**:

1. **Leaderboard Updates**: O(n log n) for sorting all players
   - **Solution**: Cache leaderboard, update periodically (not per transaction)
   - **Implementation**: Only update after significant events (fee distributions, predictions)

2. **Prediction Resolution**: O(n) for resolving all player predictions in a period
   - **Solution**: Lazy resolution (resolve when queried)
   - **Current**: Hybrid approach (batch resolve on price update, on-demand otherwise)

3. **Guild Member Iteration**: O(m) where m = guild size
   - **Solution**: Cap guild size (currently unlimited)
   - **Recommendation**: Add max_guild_members config parameter
