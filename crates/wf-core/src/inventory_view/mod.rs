use std::collections::BTreeMap;

use serde::Serialize;
use wf_data::{Rarity, Refinement, catch_grade, misc_item_name};

use crate::catalog::{Catalog, VaultStatus, display_name_from_path, part_name, refinement_name};
use crate::identity::unlisted_upgrade;
use crate::listings::PlacedOrders;
use crate::prices::Prices;
use crate::view::View;

mod misc;
mod parts;
mod relics;
mod upgrades;

pub(crate) use misc::misc;
pub(crate) use parts::{complete_sets, held_parts, parts, sets};
pub(crate) use relics::relics;
pub(crate) use upgrades::{UpgradeKind, arcanes, mods, upgrade_kind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct ItemStatus {
    pub built: bool,
    pub mastered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct PartSet {
    pub name: String,
    pub complete: bool,
    pub orders: PlacedOrders,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct PartRow {
    pub name: String,
    pub unique_name: String,
    pub image_name: Option<String>,
    pub count: i64,
    pub prices: Prices,
    pub set: PartSet,
    pub vault: Option<VaultStatus>,
    pub item: ItemStatus,
    pub prime: bool,
    pub market_slug: String,
    pub favourite: bool,
    pub orders: PlacedOrders,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct UpgradePrices {
    pub sell: Option<f64>,
    pub sell_max_rank: Option<f64>,
    pub is_floor: bool,
    pub buy: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct ModHolder {
    pub item_id: String,
    pub name: String,
    pub custom_name: Option<String>,
    pub image_name: Option<String>,
    pub rank: Option<u32>,
    pub takes_orokin_reactor: bool,
    pub orokin_upgrade: bool,
    pub exilus_adapter: bool,
    pub configs: Vec<usize>,
    pub forma: u32,
    pub archon_shards: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct ModRow {
    pub name: String,
    pub unique_name: String,
    pub image_name: Option<String>,
    pub count: i64,
    pub rank: Option<u32>,
    pub max_rank: Option<u32>,
    pub prices: UpgradePrices,
    pub rarity: Option<Rarity>,
    pub prime: bool,
    pub equipped_in: Vec<ModHolder>,
    pub market_slug: String,
    pub favourite: bool,
    pub orders: PlacedOrders,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct RelicRow {
    pub relic: String,
    pub tier: String,
    pub refinement: Refinement,
    pub image_name: Option<String>,
    pub count: i64,
    pub vault: VaultStatus,
    pub unique_name: String,
    pub market_slug: String,
    pub plat: Option<f64>,
    pub favourite: bool,
    pub orders: PlacedOrders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct SculptureStars {
    pub amber_filled: u32,
    pub cyan_filled: u32,
    pub amber_sockets: u32,
    pub cyan_sockets: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct MiscRow {
    pub name: String,
    pub unique_name: String,
    pub image_name: Option<String>,
    pub count: i64,
    pub ducats: Option<u32>,
    pub plat: Option<f64>,
    pub market_slug: String,
    pub market_subtype: Option<String>,
    pub stars: Option<SculptureStars>,
    pub favourite: bool,
    pub orders: PlacedOrders,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct SetComponent {
    pub unique_name: String,
    pub name: String,
    pub image_name: Option<String>,
    pub market_slug: String,
    pub owned: i64,
    pub required: i64,
    pub enough: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct SetRow {
    pub set_name: String,
    pub unique_name: String,
    pub image_name: Option<String>,
    pub owned_parts: usize,
    pub total_parts: usize,
    pub count: i64,
    pub complete: bool,
    pub item: ItemStatus,
    pub vault: Option<VaultStatus>,
    pub prices: Prices,
    pub market_slug: String,
    pub favourite: bool,
    pub orders: PlacedOrders,
    pub components: Vec<SetComponent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct TabTotals {
    pub ducats: i64,
    pub plat: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS), ts(export))]
pub struct InventoryTab {
    pub parts: Vec<PartRow>,
    pub mods: Vec<ModRow>,
    pub arcanes: Vec<ModRow>,
    pub relics: Vec<RelicRow>,
    pub misc: Vec<MiscRow>,
    pub sets: Vec<SetRow>,
    pub totals: BTreeMap<String, TabTotals>,
}

pub(crate) fn tab(view: &View) -> InventoryTab {
    let parts = parts(view);
    let mods = mods(view);
    let arcanes = arcanes(view);
    let relics = relics(view);
    let misc = misc(view);
    let sets = sets(view);

    let mut totals = BTreeMap::new();
    totals.insert(
        "parts".to_owned(),
        sum(parts
            .iter()
            .map(|row| (row.count, row.prices.ducats, row.prices.sell))),
    );
    totals.insert(
        "mods".to_owned(),
        sum(mods.iter().map(|row| (row.count, None, row.prices.sell))),
    );
    totals.insert(
        "arcanes".to_owned(),
        sum(arcanes.iter().map(|row| (row.count, None, row.prices.sell))),
    );
    totals.insert(
        "relics".to_owned(),
        sum(relics.iter().map(|row| (row.count, None, row.plat))),
    );
    totals.insert(
        "misc".to_owned(),
        sum(misc.iter().map(|row| (row.count, row.ducats, row.plat))),
    );
    totals.insert(
        "sets".to_owned(),
        sum(sets
            .iter()
            .map(|row| (row.count, row.prices.ducats, row.prices.sell))),
    );

    InventoryTab {
        parts,
        mods,
        arcanes,
        relics,
        misc,
        sets,
        totals,
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "item counts stay far below 2^53"
)]
fn sum(rows: impl Iterator<Item = (i64, Option<u32>, Option<f64>)>) -> TabTotals {
    let mut ducats = 0;
    let mut plat = 0.0;
    for (count, row_ducats, row_plat) in rows {
        ducats += i64::from(row_ducats.unwrap_or(0)) * count;
        plat += row_plat.unwrap_or(0.0) * count as f64;
    }
    TabTotals { ducats, plat }
}

pub(crate) fn catalogued_name(catalog: &Catalog, unique_name: &str) -> Option<String> {
    if let Some((relic, refinement)) = catalog.relic_by_unique_name(unique_name) {
        return Some(format!(
            "{} Relic ({})",
            relic.name,
            refinement_name(refinement)
        ));
    }
    if let Some((base, grade)) = catch_grade(unique_name)
        && let Some(item) = catalog.item(&base)
    {
        return Some(format!("{} ({grade})", item.name));
    }
    if let Some(item) = catalog.item(unique_name) {
        return Some(item.name.clone());
    }
    if let Some((item, component)) = catalog.component(unique_name) {
        return Some(part_name(item, component));
    }
    if let Some(upgrade) = unlisted_upgrade(unique_name) {
        return Some(upgrade.name.to_owned());
    }
    misc_item_name(unique_name).map(str::to_owned)
}

pub(crate) fn display_name(catalog: &Catalog, unique_name: &str) -> String {
    let stocked = unique_name
        .strip_prefix("/Lotus/StoreItems/")
        .map(|tail| format!("/Lotus/{tail}"));
    let unique_name = stocked.as_deref().unwrap_or(unique_name);
    catalogued_name(catalog, unique_name).unwrap_or_else(|| display_name_from_path(unique_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{fixtures, names_a_prime};
    use crate::prices::FixedPrices;
    use crate::view::Fixture;

    pub(super) fn prices() -> FixedPrices {
        FixedPrices::new([
            ("trinity_prime_systems_blueprint", 14.0),
            ("braton_prime_barrel", 8.0),
            ("braton_prime_set", 45.0),
            ("axi_a21_relic", 5.0),
        ])
    }

    #[test]
    fn vault_pill() {
        let stocked = Fixture::new(
            fixtures::catalog(),
            fixtures::inventory_owning(&[(
                "/Lotus/Types/Recipes/WarframeRecipes/TrinityPrimeSystemsBlueprint",
                1,
            )]),
        )
        .with_prices(prices());
        let row = &parts(&stocked.view())[0];
        assert!(row.vault.is_some());

        let mod_rows = mods(
            &Fixture::new(fixtures::catalog(), fixtures::inventory())
                .with_prices(prices())
                .view(),
        );
        assert!(!mod_rows.is_empty());

        let sets = sets(&stocked.view());
        assert!(
            sets.iter()
                .all(|row| row.vault.is_some() == names_a_prime(&row.set_name))
        );
    }

    #[test]
    fn tab_totals() {
        let fixture =
            Fixture::new(fixtures::catalog(), fixtures::inventory()).with_prices(prices());
        let tab = tab(&fixture.view());
        assert!(!tab.misc.is_empty());
        assert_eq!(tab.totals.len(), 6);
        let relics = tab.totals.get("relics").unwrap();
        assert!(relics.plat >= 5.0);
        let parts = tab.totals.get("parts").unwrap();
        assert!(parts.ducats >= 0);
    }

    fn favourite_fixture() -> wf_inventory::Inventory {
        fixtures::inventory_owning(&[
            (
                "/Lotus/Types/Recipes/Weapons/WeaponParts/BratonPrimeBarrel",
                3,
            ),
            (
                "/Lotus/Types/Recipes/Weapons/WeaponParts/BratonPrimeStock",
                1,
            ),
            (
                "/Lotus/Types/Recipes/WarframeRecipes/TrinityPrimeSystemsBlueprint",
                1,
            ),
        ])
    }

    #[test]
    fn favourites_across_tab() {
        let mut fixture =
            Fixture::new(fixtures::catalog(), favourite_fixture()).with_prices(prices());
        let plain = tab(&fixture.view());
        for rows in [&plain.mods, &plain.arcanes] {
            assert!(!rows.is_empty());
            assert!(rows.iter().all(|row| !row.favourite));
        }
        assert!(plain.parts.iter().all(|row| !row.favourite));
        assert!(plain.relics.iter().all(|row| !row.favourite));
        assert!(plain.misc.iter().all(|row| !row.favourite));
        assert!(plain.sets.iter().all(|row| !row.favourite));

        fixture.favourites = [
            plain.parts[0].unique_name.clone(),
            plain.mods[0].unique_name.clone(),
            plain.arcanes[0].unique_name.clone(),
            plain.relics[0].unique_name.clone(),
            plain.misc[0].unique_name.clone(),
        ]
        .into_iter()
        .collect();
        let marked = tab(&fixture.view());
        assert!(marked.parts[0].favourite);
        assert!(marked.mods[0].favourite);
        assert!(marked.arcanes[0].favourite);
        assert!(marked.relics[0].favourite);
        assert!(marked.misc[0].favourite);
        assert_eq!(
            marked.relics.iter().filter(|row| row.favourite).count(),
            1,
            "a favourited relic stack leaves the other refinements alone"
        );
        assert_eq!(
            marked.misc.iter().filter(|row| row.favourite).count(),
            1,
            "a favourited misc item marks one row"
        );
    }

    #[test]
    fn favourite_set_marks_parts() {
        let mut fixture =
            Fixture::new(fixtures::catalog(), favourite_fixture()).with_prices(prices());
        let set = sets(&fixture.view())
            .into_iter()
            .find(|row| row.set_name == "Braton Prime")
            .expect("Braton Prime");
        assert!(!set.favourite);

        fixture.favourites = [set.unique_name.clone()].into_iter().collect();
        let marked = sets(&fixture.view())
            .into_iter()
            .find(|row| row.set_name == "Braton Prime")
            .expect("Braton Prime");
        assert!(marked.favourite);

        let rows = parts(&fixture.view());
        let owned_parts: Vec<&PartRow> = rows
            .iter()
            .filter(|row| row.set.name == "Braton Prime")
            .collect();
        assert!(!owned_parts.is_empty());
        assert!(owned_parts.iter().all(|row| row.favourite));
        assert!(
            rows.iter()
                .any(|row| row.set.name != "Braton Prime" && !row.favourite)
        );
    }
}
