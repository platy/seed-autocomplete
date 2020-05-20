#![allow(clippy::wildcard_imports)]

use country_search::*;
use seed::app::message_mapper::MessageMapper;
use seed::{prelude::*, *};
use seed_autocomplete as autocomplete;

mod country_search;

#[derive(Default)]
struct Model {
    /// Model for the autocomplete component
    country_autocomplete: autocomplete::Model<Country>,
    /// data source for looking up suggestions, here the data is locally stored, you could instead fetch from a web service
    country_search: CountrySearch,
    selected_country: Option<celes::Country>,
}

#[derive(Clone)]
enum Msg {
    /// Wraps messages addressed to the autocomplete component
    CountryAutocomplete(autocomplete::Msg),
    /// Autocomplete notifies us that the search contents have changed so we can update the suggestions
    CountryInputChange(autocomplete::InputChanged),
    /// Autocomplete notifies us that the user has made a selection
    CountrySelected(autocomplete::SuggestionSelected),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::CountryInputChange(autocomplete::InputChanged(value)) => {
            if value.len() >= 1 {
                let suggestions = model.country_search.prefix_lookup(&value);
                model.country_autocomplete.set_suggestions(suggestions);
            }
        }
        Msg::CountrySelected(autocomplete::SuggestionSelected) => {
            let selection = model.country_autocomplete.get_selection();
            if let Some(Country(selection)) = selection.cloned() {
                model
                    .country_autocomplete
                    .set_input_value(selection.long_name.clone());
                model.selected_country = Some(selection);
            }
        }
        Msg::CountryAutocomplete(msg) => autocomplete::update(
            msg,
            &mut model.country_autocomplete,
            &mut orders.proxy(Msg::CountryAutocomplete),
        ),
    }
}

fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![
        div![
            "Search for a country name, alias or ISO 3166-1 code:",
            // the view for the autocomplete box, adding it into the vdom
            autocomplete::view(&model.country_autocomplete).map_msg(Msg::CountryAutocomplete),
        ]
        if let Some(selected_country) = &model.selected_country {
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
        } else {
            div![]
        }
    ]
}

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders.subscribe(Msg::CountryInputChange);
    orders.subscribe(Msg::CountrySelected);
    AfterMount::default()
}

#[wasm_bindgen(start)]
pub fn start() {
    App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
