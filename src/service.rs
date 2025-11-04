#![cfg_attr(target_arch = "wasm32", no_main)]

use std::sync::Arc;

use async_graphql::{EmptySubscription, Object, Schema};
use linera_sdk::{
    graphql::GraphQLMutationRoot,
    linera_base_types::{Amount, WithServiceAbi},
    views::View,
    Service, ServiceRuntime,
};

use predictive_manager::state::*;
use predictive_manager::Operation;

// Wrapper to pass state reference through GraphQL context
// We use a raw pointer because GraphQL requires 'static lifetime, but we know
// the query execution completes within handle_query, so the reference is valid.
// This is safe because:
// 1. The state outlives the query execution (it's owned by self)
// 2. handle_query is async but the query executes synchronously within it
// 3. No concurrent access to state during query execution
struct StateWrapper {
    state: *const PredictionMarketState,
}

unsafe impl Send for StateWrapper {}
unsafe impl Sync for StateWrapper {}

impl StateWrapper {
    // Safety: This is safe because:
    // - The pointer is created from a valid reference to self.state
    // - The query execution completes before handle_query returns
    // - No mutable access to state occurs during query execution
    #[inline]
    unsafe fn state(&self) -> &PredictionMarketState {
        &*self.state
    }
}

pub struct PredictiveManagerService {
    state: PredictionMarketState,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(PredictiveManagerService);

impl WithServiceAbi for PredictiveManagerService {
    type Abi = predictive_manager::PredictiveManagerAbi;
}

impl Service for PredictiveManagerService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = PredictionMarketState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        PredictiveManagerService {
            state,
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, query: Self::Query) -> Self::QueryResponse {
        // Pass state access through GraphQL data context using a raw pointer
        // Safe because query execution completes within this method
        let state_wrapper = StateWrapper {
            state: &self.state as *const PredictionMarketState,
        };

        Schema::build(
            QueryRoot,
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .data(state_wrapper)
        .finish()
        .execute(query)
        .await
    }
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get total supply of points
    async fn total_supply(&self, ctx: &async_graphql::Context<'_>) -> Amount {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        *state.total_supply.get()
    }

    /// Get total points earned by a player (mirrors contract's get_player_total_points)
    async fn player_total_points(
        &self,
        ctx: &async_graphql::Context<'_>,
        player_id: PlayerId,
    ) -> async_graphql::Result<Amount> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        let player = state
            .players
            .get(&player_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Player not found"))?;
        Ok(player.total_earned)
    }

    /// Get total points earned by all members of a guild (mirrors contract's get_guild_total_points)
    async fn guild_total_points(
        &self,
        ctx: &async_graphql::Context<'_>,
        guild_id: GuildId,
    ) -> async_graphql::Result<Amount> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        let guild = state
            .guilds
            .get(&guild_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Guild not found"))?;

        let mut total_guild_points = Amount::ZERO;

        // Sum total points earned by all guild members (same logic as contract's get_guild_total_points)
        for member_id in &guild.members {
            if let Some(member) = state.players.get(member_id).await? {
                total_guild_points = total_guild_points.saturating_add(member.total_earned);
            }
        }

        Ok(total_guild_points)
    }

    /// Get all guilds that have been created (mirrors contract's get_all_guilds)
    async fn all_guilds(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<Guild>> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        let mut guilds = Vec::new();
        state
            .guilds
            .for_each_index_value(|_guild_id, guild| {
                // guild is Cow<'_, Guild>, convert to owned Guild
                guilds.push(guild.into_owned());
                Ok(())
            })
            .await
            .map_err(|e| async_graphql::Error::new(format!("Failed to iterate guilds: {:?}", e)))?;
        Ok(guilds)
    }

    /// Get all members in a specific guild (mirrors contract's get_guild_members)
    async fn guild_members(
        &self,
        ctx: &async_graphql::Context<'_>,
        guild_id: GuildId,
    ) -> async_graphql::Result<Vec<Player>> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        let guild = state
            .guilds
            .get(&guild_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Guild not found"))?;

        let mut members = Vec::new();

        // Get all player data for guild members (same logic as contract's get_guild_members)
        for member_id in &guild.members {
            if let Some(player) = state.players.get(member_id).await? {
                members.push(player.clone());
            }
        }

        Ok(members)
    }

    /// Get a player by their ID (mirrors contract's get_player)
    async fn player(
        &self,
        ctx: &async_graphql::Context<'_>,
        player_id: PlayerId,
    ) -> async_graphql::Result<Player> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        state
            .players
            .get(&player_id)
            .await?
            .map(|p| p.clone())
            .ok_or_else(|| async_graphql::Error::new("Player not found"))
    }

    /// Get a market by its ID (mirrors contract's get_market)
    async fn market(
        &self,
        ctx: &async_graphql::Context<'_>,
        market_id: MarketId,
    ) -> async_graphql::Result<Market> {
        let state_wrapper = ctx.data_unchecked::<StateWrapper>();
        let state = unsafe { state_wrapper.state() };
        state
            .markets
            .get(&market_id)
            .await?
            .map(|m| m.clone())
            .ok_or_else(|| async_graphql::Error::new("Market not found"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::{Request, Response, Value};
    use futures::FutureExt as _;
    use linera_sdk::{
        linera_base_types::Amount, util::BlockingWait, views::View, Service, ServiceRuntime,
    };
    use serde_json::json;

    use super::{PredictionMarketState, PredictiveManagerService};

    #[test]
    fn query() {
        let total_supply = Amount::from_tokens(100);
        let runtime = Arc::new(ServiceRuntime::<PredictiveManagerService>::new());
        let mut state = PredictionMarketState::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");
        state.total_supply.set(total_supply);

        let service = PredictiveManagerService { state, runtime };
        let request = Request::new("{ totalSupply }");

        let response = service
            .handle_query(request)
            .now_or_never()
            .expect("Query should not await anything");

        let expected = Response::new(Value::from_json(json!({"totalSupply": "100."})).unwrap()); // the value go exceeds

        assert_eq!(response, expected)
    }
}
