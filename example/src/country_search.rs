//! Loads country data from the [celes](https://crates.io/crates/celes) crate with each of the ISO 3166-1 ways of referring to a country forming keys in a [ternary search tree](https://crates.io/crates/tst), allowing for prefix searches
use std::collections::BTreeSet;
use std::fmt;
use tst::TSTMap;

pub struct CountrySearch {
    /// prefixes for all the countries' different representations, pointing to their index
    prefixes: TSTMap<Vec<usize>>,
    entries: Vec<celes::Country>,
}

impl Default for CountrySearch {
    fn default() -> CountrySearch {
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

        CountrySearch { entries, prefixes }
    }
}

/// wrap `celes::Country` in new type to reimplement Display without removing spaces
#[derive(Clone, Debug)]
pub struct Country(pub celes::Country);

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.long_name)
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
            .map(|idx| Country(self.entries[idx].clone()))
            .collect()
    }
}
