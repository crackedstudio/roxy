```
# 🦝 Roxy Smart Contracts

Roxy is a **fully on-chain, real-time crypto prediction and portfolio management game** built on the **Linera blockchain**.  
Each player manages a portfolio, predicts market outcomes, and competes in ranked tournaments — all powered by Linera’s **microchain architecture** for speed, scalability, and decentralization.

> 🎮 **Frontend:** [Roxy Client Repo →](https://github.com/crackedstudio/roxy-client)

---

## 🚀 What It Does

Roxy turns decentralized prediction markets into a **fun, gamified, and social experience**.  
Players can:

-   🧩 **Predict Multi-Outcome Events** — from crypto price trends to sports results
-   ⚡ **Trade in Real-Time** — with dynamic AMM-style pricing based on live demand
-   🏆 **Earn Through Skill** — accurate predictions yield token rewards
-   🧗 **Progress and Level Up** — gain XP, unlock badges, and climb leaderboards
-   🛡️ **Form Guilds** — team up to strategize and share rewards
-   🧠 **Create Markets** — launch your own markets and earn creator fees
-   🗳️ **Vote as Oracles** — use reputation scores to resolve market outcomes

Market types include:

-   Quick predictions (minutes/hours)
-   Tournament brackets (days)
-   Seasonal events (weeks/months)
-   Direct PvP challenges

Each player and market runs on its **own microchain**, synchronized via the **Master Game Chain**, ensuring real-time responsiveness and massive scalability.

---

## 🧩 The Problem It Solves

### 1. Complex Prediction Markets

Traditional platforms like Augur and Polymarket feel technical and inaccessible.  
**Roxy** simplifies this by blending gaming and DeFi — making prediction markets **as fun as an arcade**.

### 2. Poor Engagement & Retention

Existing markets lack:

-   Progression systems
-   Social features
-   Emotional reward loops

Roxy adds **XP, guilds, leaderboards**, and **visual game design** to make users stay.

### 3. Scalability Bottlenecks

Roxy leverages Linera’s microchains:

-   Personal chains for each player (no congestion)
-   Parallel processing for all markets
-   Fee-less transactions
-   Instant finality

### 4. The Oracle Problem

Roxy uses **community oracle voting** with:

-   Reputation-weighted votes
-   Hybrid data + community consensus
-   Staking incentives for accuracy

### 5. Broken Tokenomics

Roxy creates a **circular economy**:

-   Earn tokens via predictions
-   Spend tokens on trades or market creation
-   Market creators earn fees
-   Guilds share collective rewards

---

## ⚙️ Technologies Used

### **Blockchain & Smart Contracts**

-   **Linera Blockchain** — scalable microchain-based L1
-   **Rust** — smart contract language
-   **linera-sdk** — Linera application framework
-   **linera-views** — storage abstractions (MapView, RegisterView, RootView)

### **Smart Contract Components**

-   `RootView` — global app state
-   `MapView` — market/player/guild storage
-   `RegisterView` — configuration storage
-   `Operations` — synchronous user actions
-   `Messages` — asynchronous cross-chain communications

### **AMM & Token Economics**

-   Simplified bonding curve (`price = base_price × (supply / liquidity)^factor`)
-   Proportional payout and slippage protection
-   Dynamic fee and reward distribution

### **Oracle System**

-   Reputation-weighted voting
-   Quorum and aggregation logic
-   Hybrid on-chain + community resolution

---

## 🏗️ How We Built It

### **Phase 1 — Architecture Design**

-   Studied Linera microchain patterns and messaging
-   Designed models for markets, players, guilds, and oracles
-   Mapped cross-chain flows (player onboarding, market resolution, leaderboards)

### **Phase 2 — Smart Contract Development**

#### 1. Master Game Application

-   Global config, market registry, leaderboard
-   Player registry, XP and achievement logic

#### 2. Market Application

-   Market states, outcome tracking, resolution logic
-   Position recording and reward distribution

#### 3. Player Application

-   Points, stats, reputation, achievements
-   Prediction history and leaderboard sync

#### 4. Oracle Application

-   Vote collection, reputation scoring, and resolution triggers

### **Phase 3 — Core Features**

-   Points & reward mechanics
-   Real-time market participation
-   Level and XP progression
-   Oracle-based resolution

### **Phase 4 — Cross-Chain Integration**

-   Asynchronous messaging between microchains
-   Event synchronization and error handling

### **Phase 5 — Testing & Optimization**

-   Unit and integration testing for each module
-   Validation of market resolution and reward logic

---

## 🧠 Challenges We Faced

1. **Learning Linera’s Novel Architecture**

    - Microchains required a new way of thinking about state and coordination.
    - Solved with modular separation: `Master → Market → Player` chains.

2. **Simplifying AMM Pricing for Gaming**

    - Replaced heavy math with an intuitive bonding curve.
    - Easy to balance gameplay while staying economically sound.

3. **Oracle Resolution at Scale**
    - Implemented a hybrid approach combining automated feeds and community voting.

---

## 🎓 What We Learned

-   Fully on-chain apps can be **fast, interactive, and scalable** with microchains.
-   **Cross-chain state** design demands careful message orchestration.
-   Game mechanics can thrive directly **on-chain** without central servers.

---

## 🔮 What’s Next for Roxy

1. **Frontend Development**

    - React-based UI with Linera wallet support
    - Real-time updates via WebSocket
    - Mobile-optimized gameplay

2. **Expanded Game Modes**

    - Battle Royale predictions
    - Guild vs Guild tournaments
    - Mystery Markets & Speed Rounds

3. **AI Integration**

    - AI-powered market prediction assistants
    - Autonomous oracle agents

4. **Mainnet Launch**
    - Deploy Roxy on Linera mainnet
    - Community tournaments and seasonal leaderboards

---

## 🏛️ Contract Architecture

Roxy's smart contract system is built on **Linera's microchain architecture** using **Rust** and the **linera-sdk**. The codebase is organized into modular components that handle different aspects of the prediction market game.

### **Core Contract Structure**

The main contract is implemented in `src/contract.rs` as `PredictionMarketContract`, which serves as the central orchestrator for all game operations.

#### **Key Components:**

```rust
pub struct PredictionMarketContract {
    state: PredictionMarketState,
    runtime: ContractRuntime<Self>,
}
```

### **State Management (`src/state.rs`)**

The application state is managed through Linera's `RootView` pattern, providing efficient storage and retrieval:

```rust
#[derive(RootView)]
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
}
```

**Storage Types:**

-   **`RegisterView`** — Single values (config, leaderboard, counters)
-   **`MapView`** — Key-value mappings (players, markets, guilds)

### **Data Models**

#### **Core Entities:**

1. **`Player`** — Player profiles with progression data

    - Token balance, XP, level, reputation
    - Market participation history
    - Guild membership and achievements

2. **`Market`** — Prediction market instances

    - Multiple outcomes with dynamic pricing
    - AMM-style liquidity pools
    - Resolution methods (Oracle, Automated, Creator)

3. **`Guild`** — Social groups for collaborative gameplay

    - Member management and shared pools
    - Collective profit tracking

4. **`Achievement`** — Progression rewards system
    - XP and token rewards
    - Various requirement types

### **Operation Handlers (`src/lib.rs`)**

The contract exposes operations through a comprehensive enum:

```rust
pub enum Operation {
    // Player operations
    RegisterPlayer { display_name: Option<String> },
    UpdateProfile { display_name: Option<String> },
    ClaimDailyReward,

    // Market operations
    CreateMarket { title: String, description: String, ... },
    BuyShares { market_id: MarketId, outcome_id: OutcomeId, ... },
    SellShares { market_id: MarketId, outcome_id: OutcomeId, ... },

    // Voting operations
    VoteOnOutcome { market_id: MarketId, outcome_id: OutcomeId },
    TriggerResolution { market_id: MarketId },
    ClaimWinnings { market_id: MarketId },

    // Guild operations
    CreateGuild { name: String },
    JoinGuild { guild_id: GuildId },
    LeaveGuild,
    ContributeToGuild { amount: Amount },

    // Admin operations
    UpdateGameConfig { config: GameConfig },
}
```

### **Core Game Logic**

#### **1. Player Management**

-   **Registration**: New players receive initial tokens and XP
-   **Progression**: Level-up system with XP thresholds
-   **Daily Rewards**: 24-hour cooldown token distribution
-   **Achievement System**: Automated reward distribution

#### **2. Market Operations**

-   **Creation**: Players pay creation costs to launch markets
-   **Trading**: AMM-style pricing with slippage protection
-   **Resolution**: Multiple resolution methods (Oracle voting, automated, creator-decided)
-   **Payouts**: Proportional distribution to winning positions

#### **3. AMM Pricing Model**

The contract implements a simplified bonding curve for market pricing:

```rust
// AMM Formula: Share_Price = Base_Price × (Current_Shares_Sold / Total_Supply)^smoothing_factor
fn calculate_shares_for_amount(&self, market: &Market, outcome_id: OutcomeId, amount: Amount) -> Result<Amount, ContractError>
```

#### **4. Oracle System**

-   **Reputation-weighted voting** for market resolution
-   **Quorum requirements** to ensure consensus
-   **Time-bounded voting periods**

#### **5. Guild System**

-   **Social features** for collaborative gameplay
-   **Shared token pools** for collective investments
-   **Guild leaderboards** and profit sharing

### **Error Handling**

Comprehensive error types cover all failure scenarios:

```rust
pub enum ContractError {
    Unauthorized,
    PlayerAlreadyExists,
    DailyRewardAlreadyClaimed,
    InvalidOutcomeCount,
    DurationTooShort,
    InsufficientBalance,
    MarketNotActive,
    MarketEnded,
    // ... and many more
}
```

### **Cross-Chain Messaging**

The contract uses Linera's messaging system for cross-chain communication:

```rust
pub enum Message {
    MarketCreated { market_id: MarketId, creator: PlayerId },
    MarketResolved { market_id: MarketId, winning_outcome: OutcomeId },
    TradeExecuted { player_id: PlayerId, market_id: MarketId, ... },
    PlayerLeveledUp { player_id: PlayerId, new_level: u32 },
    AchievementUnlocked { player_id: PlayerId, achievement_id: AchievementId },
    GuildCreated { guild_id: GuildId, name: String },
}
```

### **Service Layer (`src/service.rs`)**

The GraphQL service provides query capabilities:

```rust
pub struct PredictiveManagerService {
    state: PredictionMarketState,
    runtime: Arc<ServiceRuntime<Self>>,
}
```

**Features:**

-   **GraphQL API** for frontend integration
-   **Real-time queries** for market data
-   **Player statistics** and leaderboards

### **Key Design Patterns**

1. **Modular Architecture** — Clear separation of concerns
2. **Immutable State Updates** — All state changes through controlled operations
3. **Event-Driven Design** — Cross-chain messaging for coordination
4. **Economic Incentives** — Token-based reward system
5. **Scalable Storage** — Linera's efficient view system

### **Security Considerations**

-   **Access Control** — Admin-only configuration updates
-   **Slippage Protection** — Price limits on trades
-   **Input Validation** — Comprehensive parameter checking
-   **Economic Safety** — Overflow protection with `saturating_*` operations
-   **Oracle Security** — Reputation-weighted voting with quorum requirements

This architecture enables Roxy to scale efficiently across Linera's microchain network while maintaining security and providing a rich gaming experience.

---

### 🦝 Built with ❤️ by Cracked Studio

Roxy — where **crypto strategy meets real-time gameplay.**
```