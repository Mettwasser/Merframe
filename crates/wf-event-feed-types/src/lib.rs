pub mod market;

use chrono::{DateTime, Utc};
use serde::Serialize;
use wf_core::{FissureInfo, InventorySummary, RewardScreen, Trade};

use crate::market::MarketAutoClose;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FeedStatus {
    pub game_detected: bool,
    pub scanning: bool,
    pub log_attached: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub world_state_at: Option<DateTime<Utc>>,
    pub trades_remaining: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FeedInventory {
    pub mr: u32,
    pub plat: i64,
    pub credits: i64,
    pub endo: i64,
    pub ducats: i64,
    pub changes: usize,
}

impl From<&InventorySummary> for FeedInventory {
    fn from(summary: &InventorySummary) -> Self {
        Self {
            mr: summary.mr,
            plat: summary.plat,
            credits: summary.credits,
            endo: summary.endo,
            ducats: summary.ducats,
            changes: summary.changes,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum FeedEvent {
    Status(FeedStatus),
    InventoryUpdated(FeedInventory),
    RelicRewardScreen {
        relic: Option<String>,
        screen: RewardScreen,
    },
    TradeCompleted {
        at: DateTime<Utc>,
        trade: Trade,
    },
    FissureAlert {
        fissure: FissureInfo,
    },
    TimerAlert {
        name: String,
        next_state: String,
        ends_at: DateTime<Utc>,
        remaining_secs: i64,
    },
    TradesUpdated,
    WorldStateUpdated,
    RivenDataUpdated,
    PricesUpdated,
    FavouriteUpdated,
    MarketAutoClosed(MarketAutoClose),
}
