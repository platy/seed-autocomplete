//! Loads country data from the [celes](https://crates.io/crates/celes) crate with each of the ISO 3166-1 ways of referring to a country forming keys in a [ternary search tree](https://crates.io/crates/tst), allowing for prefix searches
pub use celes::Country;
use std::collections::BTreeSet;
use tst::TSTMap;

pub struct CountrySearch {
    /// prefixes for all the countries' different representations, pointing to their index
    prefixes: TSTMap<Vec<usize>>,
    entries: Vec<celes::Country>,
}

impl Default for CountrySearch {
    fn default() -> Self {
        let entries = celes::Country::get_countries();
        let mut prefixes = TSTMap::new();

        for (
            idx,
            celes::Country {
                code,
                value: _,
                alpha2,
                alpha3,
                long_name,
                aliases,
            },
        ) in entries.iter().cloned().enumerate()
        {
            for key in [code, alpha2, alpha3, long_name]
                .iter()
                .chain(aliases.iter())
            {
                prefixes
                    .entry(&key.to_lowercase())
                    .or_insert(vec![])
                    .push(idx);
            }
        }

        Self { entries, prefixes }
    }
}

impl CountrySearch {
    pub fn prefix_lookup(&self, prefix: &str) -> Vec<Country> {
        self.prefixes
            .prefix_iter(&prefix.to_lowercase())
            .flat_map(|(_k, v)| v)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|idx| self.entries[idx].clone())
            .collect()
    }
}
