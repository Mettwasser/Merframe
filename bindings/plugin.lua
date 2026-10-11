---@meta
-- Merframe plugin API type definitions.
--
-- Drop this file into your language server's workspace (or reference it from
-- `.luarc.json`) to get completion and type checking for the global `App` API.
--
-- Registering a listener:
--
--     App.on("trade_completed", function(data)
--         -- `data` is the externally tagged CoreEvent table, e.g.
--         local trade = data.TradeCompleted.trade
--         print(("received %d items"):format(#trade.received))
--     end)
--
-- IMPORTANT: the *event name* is snake_case (derived from the Rust
-- `CoreEvent` discriminant), while the *payload key* is the PascalCase Rust
-- variant name. Both are listed per event below.
--
-- Payloads are serialized from Rust with serde, so:
--   * `Option<T>` fields are `nil` when absent,
--   * `DateTime<Utc>` becomes an RFC 3339 string (see `DateTime`),
--   * tuples become 1-based arrays (see `LevelRange`).

-- ---------------------------------------------------------------------------
-- Scalars
-- ---------------------------------------------------------------------------

---An RFC 3339 / ISO 8601 UTC timestamp, e.g. `"2026-10-11T18:05:00.123456789Z"`.
---@alias DateTime string

---An inclusive `(min, max)` level range serialized as a 1-based array.
---@alias LevelRange [integer, integer]

---Vault state of a Prime item.
---@alias VaultStatus
---| "vaulted"   # currently in the Prime Vault
---| "available" # currently obtainable
---| "unknown"   # vault status not known

-- ---------------------------------------------------------------------------
-- Inventory
-- ---------------------------------------------------------------------------

---Payload for the `inventory_updated` event.
---@class InventorySummary
---@field last_sync_oid string        # id of the last synced inventory
---@field mr integer                  # mastery rank
---@field plat integer                # platinum
---@field credits integer
---@field endo integer
---@field ducats integer
---@field changes integer             # number of inventory changes since the last sync

-- ---------------------------------------------------------------------------
-- Relic / reward screen
-- ---------------------------------------------------------------------------

---A single item that can be displayed (reward, part, set, ...).
---@class ItemSummary
---@field unique_name string
---@field name string
---@field image_name string?
---@field market_slug string?         # warframe.market slug, when tradable
---@field prime boolean
---@field vault VaultStatus?
---@field ducats integer?             # Ducat value, when the item is Ducat-worth

---Per-item ownership details for a reward's backing set.
---@class RewardOwnership
---@field owned integer               # how many the player owns
---@field needed integer              # how many are required
---@field needed_for_set boolean      # this part is still missing for the set
---@field parent_owned boolean        # the parent item is owned or building

---A component that makes up the reward's set.
---@class RankedComponent
---@field item ItemSummary
---@field owned integer?              # `nil` when no inventory is loaded
---@field needed integer
---@field enough boolean              # `owned >= needed`
---@field this_reward boolean         # the component that actually dropped
---@field favourite boolean

---A reward option on a relic reward screen.
---@class RankedReward
---@field item ItemSummary
---@field plat number?                # platinum price of the item, when known
---@field set_plat number?            # platinum price of the full set, when known
---@field ownership RewardOwnership?  # `nil` when no inventory is loaded
---@field favourite boolean
---@field best boolean                # highest priced reward on the screen
---@field components RankedComponent[]

---Player currency balance shown alongside a reward screen.
---@class AccountBalance
---@field plat integer
---@field ducats integer

---Payload for the `relic_reward_screen` event.
---@class RewardScreen
---@field ranked RankedReward[]
---@field account AccountBalance?      # `nil` when no inventory is loaded

-- ---------------------------------------------------------------------------
-- Trades
-- ---------------------------------------------------------------------------

---A stack of one traded item.
---@class TradeItem
---@field name string
---@field count integer
---@field rank integer?                # mod/arcane rank; `nil` for non-rankable items

---Payload for the `trade_completed` event.
---@class Trade
---@field offered TradeItem[]          # items the player gave away
---@field received TradeItem[]         # items the player received
---@field plat integer                 # net platinum (positive = gained)

-- ---------------------------------------------------------------------------
-- World state
-- ---------------------------------------------------------------------------

---Payload for the `fissure_alert` event.
---@class FissureInfo
---@field node_id string
---@field node_name string?
---@field mission_type string
---@field mission_name string
---@field planet string?
---@field tier string                  # e.g. "Lith", "Axi", ...
---@field steel_path boolean
---@field is_storm boolean
---@field faction string?
---@field levels LevelRange?
---@field expiry DateTime
---@field remaining_secs integer

-- ---------------------------------------------------------------------------
-- CoreEvent payloads (externally tagged, keyed by Rust variant name)
-- ---------------------------------------------------------------------------

---@class InventoryUpdatedEvent
---@field InventoryUpdated InventorySummary

---@class RelicRewardScreenPayload
---@field relic string?                # the player's own relic, when known
---@field screen RewardScreen

---@class RelicRewardScreenEvent
---@field RelicRewardScreen RelicRewardScreenPayload

---@class TradeCompletedPayload
---@field at DateTime
---@field partner string?              # trading partner, when known
---@field trade Trade

---@class TradeCompletedEvent
---@field TradeCompleted TradeCompletedPayload

---@class NewConversationPayload
---@field channel string               # chat channel, e.g. "Squad"
---@field player string

---@class NewConversationEvent
---@field NewConversation NewConversationPayload

---@class FissureAlertPayload
---@field fissure FissureInfo

---@class FissureAlertEvent
---@field FissureAlert FissureAlertPayload

---@class TimerAlertPayload
---@field name string                  # e.g. "Cetus", "Vallis", ...
---@field next_state string
---@field ends_at DateTime
---@field remaining_secs integer

---@class TimerAlertEvent
---@field TimerAlert TimerAlertPayload

---Any event payload passed to a listener.
---@alias CoreEvent
---| InventoryUpdatedEvent
---| RelicRewardScreenEvent
---| TradeCompletedEvent
---| NewConversationEvent
---| FissureAlertEvent
---| TimerAlertEvent

-- ---------------------------------------------------------------------------
-- API
-- ---------------------------------------------------------------------------

---Event names accepted by `App.on`, derived from `CoreEvent` (snake_case).
---@alias CoreEventName
---| "inventory_updated"
---| "relic_reward_screen"
---| "trade_completed"
---| "new_conversation"
---| "fissure_alert"
---| "timer_alert"

---The Merframe plugin API.
---@class App
App = {}

---Register a listener for a Merframe core event.
---@param event CoreEventName
---@param callback fun(data: CoreEvent)
---@overload fun(event: "inventory_updated", callback: fun(data: InventoryUpdatedEvent))
---@overload fun(event: "relic_reward_screen", callback: fun(data: RelicRewardScreenEvent))
---@overload fun(event: "trade_completed", callback: fun(data: TradeCompletedEvent))
---@overload fun(event: "new_conversation", callback: fun(data: NewConversationEvent))
---@overload fun(event: "fissure_alert", callback: fun(data: FissureAlertEvent))
---@overload fun(event: "timer_alert", callback: fun(data: TimerAlertEvent))
function App.on(event, callback) end
