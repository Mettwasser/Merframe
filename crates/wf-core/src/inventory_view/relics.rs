use wf_data::Refinement;

use crate::catalog::{
    REFINEMENTS, VaultStatus, display_name_from_path, projection_suffix,
    refinement_from_unique_name,
};
use crate::identity::ItemKind;
use crate::listings::PlacedOrders;
use crate::view::View;

use super::RelicRow;

const PROJECTION_TIERS: [(&str, &str); 6] = [
    ("T0", "Void"),
    ("T1", "Lith"),
    ("T2", "Meso"),
    ("T3", "Neo"),
    ("T4", "Axi"),
    ("T5", "Requiem"),
];

fn projection_identity(unique_name: &str) -> (String, String) {
    let leaf = unique_name
        .rsplit_once('/')
        .map_or(unique_name, |(_, leaf)| leaf);
    let Some((tier, rest)) = PROJECTION_TIERS
        .into_iter()
        .find_map(|(code, tier)| leaf.strip_prefix(code).map(|rest| (tier, rest)))
    else {
        return (display_name_from_path(leaf), String::new());
    };
    let rest = rest.strip_prefix("VoidProjection").unwrap_or(rest);
    let designation = REFINEMENTS
        .into_iter()
        .find_map(|refinement| rest.strip_suffix(projection_suffix(refinement)))
        .unwrap_or(rest);
    if designation.is_empty() {
        return (tier.to_owned(), tier.to_owned());
    }
    (
        format!("{tier} {}", display_name_from_path(designation)),
        tier.to_owned(),
    )
}

pub(crate) fn relics(view: &View) -> Vec<RelicRow> {
    let View {
        account,
        catalog,
        items,
        prices,
        favourites,
        listings,
    } = *view;
    let inventory = &account.inventory;
    let mut rows: Vec<RelicRow> = inventory
        .relics()
        .filter(|(_, count)| *count > 0)
        .filter_map(|(unique_name, count)| {
            let known = catalog.relic_by_unique_name(unique_name);
            if known.is_some_and(|(relic, _)| !relic.tradable) {
                return None;
            }
            let (relic, tier, refinement) = if let Some((relic, refinement)) = known {
                (relic.name.clone(), relic.tier.clone(), refinement)
            } else {
                let (relic, tier) = projection_identity(unique_name);
                let refinement =
                    refinement_from_unique_name(unique_name).unwrap_or(Refinement::Intact);
                (relic, tier, refinement)
            };
            let record = items
                .get(unique_name)
                .filter(|record| matches!(record.kind, ItemKind::Relic { .. }));
            let market_slug = record.and_then(|record| record.market_slug.as_deref());
            Some(RelicRow {
                relic,
                tier,
                refinement,
                image_name: record.and_then(|record| record.image_name.clone()),
                count,
                vault: record
                    .and_then(|record| record.vault)
                    .unwrap_or(VaultStatus::Unknown),
                plat: market_slug.and_then(|slug| prices.plat(slug)),
                favourite: favourites.contains(unique_name),
                orders: market_slug
                    .map_or_else(PlacedOrders::default, |slug| listings.orders_for(slug)),
                unique_name: unique_name.to_owned(),
                market_slug: market_slug.unwrap_or_default().to_owned(),
            })
        })
        .collect();
    rows.sort_by(|left, right| {
        left.relic
            .cmp(&right.relic)
            .then(left.refinement.cmp(&right.refinement))
    });
    rows
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::super::tests::prices;
    use super::*;
    use crate::catalog::{Catalog, fixtures};
    use crate::view::Fixture;

    const RELIC_PROJECTIONS: &str = include_str!("../../../../fixtures/relic_projections.json");

    fn projection_catalog() -> Catalog {
        Catalog::from_json(fixtures::ITEMS, RELIC_PROJECTIONS, fixtures::COMPONENTS).unwrap()
    }

    #[test]
    fn every_owned_relic_resolves() {
        let fixture =
            Fixture::new(projection_catalog(), fixtures::inventory()).with_prices(prices());
        let rows = relics(&fixture.view());
        let derived: Vec<&str> = rows
            .iter()
            .filter(|row| {
                fixture
                    .catalog
                    .relic_by_unique_name(&row.unique_name)
                    .is_none()
            })
            .map(|row| row.unique_name.as_str())
            .collect();
        assert!(derived.is_empty(), "{derived:?}");
        assert_eq!(rows.len(), 25);
        assert!(rows.iter().any(|row| row.vault == VaultStatus::Vaulted));
        assert!(rows.iter().any(|row| row.vault == VaultStatus::Available));
    }

    #[test]
    fn lith_g12_from_sevagoth_projection() {
        let fixture =
            Fixture::new(projection_catalog(), fixtures::inventory()).with_prices(prices());
        let row = relics(&fixture.view())
            .into_iter()
            .find(|row| {
                row.unique_name
                    == "/Lotus/Types/Game/Projections/T1VoidProjectionSevagothPrimeDBronze"
            })
            .unwrap();
        assert_eq!(row.relic, "Lith G12");
        assert_eq!(row.tier, "Lith");
        assert_eq!(row.refinement, Refinement::Intact);
        assert_eq!(row.count, 25);
    }

    #[test]
    fn one_row_per_refinement() {
        let fixture =
            Fixture::new(projection_catalog(), fixtures::inventory()).with_prices(prices());
        let rows = relics(&fixture.view());
        let counted = |refinement: Refinement| {
            rows.iter()
                .filter(|row| row.refinement == refinement)
                .count()
        };
        assert_eq!(counted(Refinement::Intact), 16);
        assert_eq!(counted(Refinement::Exceptional), 3);
        assert_eq!(counted(Refinement::Flawless), 2);
        assert_eq!(counted(Refinement::Radiant), 4);

        let refined: Vec<&RelicRow> = rows.iter().filter(|row| row.relic == "Lith S18").collect();
        let refinements: Vec<Refinement> = refined.iter().map(|row| row.refinement).collect();
        assert_eq!(refinements, REFINEMENTS);
        let names: HashSet<&str> = refined.iter().map(|row| row.unique_name.as_str()).collect();
        assert_eq!(names.len(), refined.len());
    }

    #[test]
    fn untradable_relics_dropped() {
        let fixture =
            Fixture::new(projection_catalog(), fixtures::inventory()).with_prices(prices());
        let rows = relics(&fixture.view());
        assert!(
            !rows
                .iter()
                .any(|row| row.unique_name.ends_with("T5VoidProjectionImmortalOmniA")),
            "the eterna relic cannot be traded"
        );
        let requiem = rows
            .iter()
            .find(|row| row.relic == "Requiem I")
            .expect("Requiem I");
        assert_eq!(requiem.tier, "Requiem");
        assert_eq!(requiem.count, 8);
    }

    #[test]
    fn projection_identity_fallback() {
        assert_eq!(
            projection_identity(
                "/Lotus/Types/Game/Projections/T1VoidProjectionSevagothPrimeDBronze"
            ),
            (String::from("Lith Sevagoth Prime D"), String::from("Lith"))
        );
        assert_eq!(
            projection_identity("/Lotus/Types/Game/Projections/T4VoidProjectionPPlatinum"),
            (String::from("Axi P"), String::from("Axi"))
        );
        assert_eq!(
            projection_identity("/Lotus/Types/Game/Projections/T5VoidProjectionImmortalOmniA"),
            (
                String::from("Requiem Immortal Omni A"),
                String::from("Requiem")
            )
        );
    }

    #[test]
    fn axi_a21_row() {
        let fixture =
            Fixture::new(fixtures::catalog(), fixtures::inventory()).with_prices(prices());
        let rows = relics(&fixture.view());
        assert!(rows.iter().all(|row| row.count > 0));

        let known = rows
            .iter()
            .find(|row| row.vault != VaultStatus::Unknown)
            .expect("known relic");
        assert_eq!(known.relic, "Axi A21");
        assert_eq!(known.tier, "Axi");
        assert_eq!(known.refinement, Refinement::Intact);
        assert_eq!(known.count, 3);
        assert_eq!(known.vault, VaultStatus::Available);
        assert_eq!(known.plat, Some(5.0));
        assert_eq!(known.market_slug, "axi_a21_relic");
        assert!(known.image_name.is_some());
        let unknown = rows
            .iter()
            .find(|row| row.vault == VaultStatus::Unknown)
            .expect("relic outside the export");
        assert_eq!(unknown.market_slug, "");
    }
}
