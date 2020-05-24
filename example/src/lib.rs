#![deny(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]

use country_search::{Country, CountrySearch};
use seed::{prelude::*, *};
use seed_autocomplete as autocomplete;

mod country_search;

struct Model {
    /// Model for the autocomplete component
    country_autocomplete: autocomplete::Model<Msg, Country>,
    /// data source for looking up suggestions, here the data is locally stored, you could instead fetch from a web service
    country_search: CountrySearch,
    country_selected: Option<celes::Country>,
}

#[derive(Clone)]
enum Msg {
    /// Wraps messages addressed to the autocomplete component
    CountryAutocomplete(autocomplete::Msg),
    /// Autocomplete notifies us that the search contents have changed so we can update the suggestions
    CountryInputChange(String),
    /// Autocomplete notifies us that the user has made a selection
    CountrySelected,
}

fn init(_: Url, _orders: &mut impl Orders<Msg>) -> Model {
    Model {
        country_autocomplete: autocomplete::Model::new(Msg::CountryAutocomplete, |s| Msg::CountryInputChange(s.to_owned()), |_| Some(Msg::CountrySelected)),
        country_search: CountrySearch::default(),
        country_selected: None,
}
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {

        Msg::CountryInputChange(value) => {
            if !value.is_empty() {
                let suggestions = model.country_search.prefix_lookup(&value);
                model.country_autocomplete.set_suggestions(suggestions);
            }
        }
        Msg::CountrySelected => {
            let selection = model.country_autocomplete.get_selection();
            if let Some(Country(selection)) = selection.cloned() {
                model
                    .country_autocomplete
                    .set_input_value(selection.long_name.clone());
                model.country_selected = Some(selection);
            }
        }
        Msg::CountryAutocomplete(msg) => autocomplete::update(
            msg,
            &mut model.country_autocomplete,
            orders,
        ),
    }
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        section![
            div![
                "Search for a country name, alias or ISO 3166-1 code:",
                // the view for the autocomplete box, adding it into the vdom
                    autocomplete::view(&model.country_autocomplete),
            ],
            model.country_selected.as_ref().map(|selected_country| {
                div![
                    h3![&selected_country.long_name],
                    ul![
                        li!["Country code:", &selected_country.code],
                        li!["2 letter code:", &selected_country.alpha2],
                        li!["3 letter code:", &selected_country.alpha3],
                        li!["Long name:", &selected_country.long_name],
                        li![
                            "Aliases:",
                            ul![selected_country.aliases.iter().map(|alias| li![alias])]
                        ],
                    ],
                ]
            }),
        ]
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
