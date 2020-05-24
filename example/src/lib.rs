#![deny(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]

use country_search::{Country, CountrySearch};
use seed::{prelude::*, *};
use seed_autocomplete as autocomplete;
use tst::TSTSet;

mod country_search;

struct Model {
    // Search autocomplete, records your previous submissions
    /// Model for the autocomplete component
    search_autocomplete: autocomplete::Model<Msg>,
    /// data source for looking up suggestions
    search_previous: TSTSet,
    search: Option<String>,

    // Weekday autocomplete, allows chososing a weekday from prepopulated list
    /// Model for the autocomplete component
    weekday_autocomplete: autocomplete::Model<Msg>,
    /// data source for looking up suggestions, here the data is locally stored, you could instead fetch from a web service
    weekday_search: TSTSet,
    weekday_selected: Option<String>,

    /// Model for the autocomplete component
    country_autocomplete: autocomplete::Model<Msg, Country>,
    /// data source for looking up suggestions, here the data is locally stored, you could instead fetch from a web service
    country_search: CountrySearch,
    country_selected: Option<celes::Country>,
}

#[derive(Clone)]
enum Msg {
    /// Wraps messages addressed to the autocomplete component
    SearchAutocomplete(autocomplete::Msg),
    /// Autocomplete notifies us that the search contents have changed so we can update the suggestions
    SearchInputChange(String),
    /// Autocomplete notifies us that the user has made a selection or entered a new search
    SearchSubmitted(String),

    /// Wraps messages addressed to the autocomplete component
    WeekdayAutocomplete(autocomplete::Msg),
    /// Autocomplete notifies us that the search contents have changed so we can update the suggestions
    WeekdayInputChange(String),
    /// Autocomplete notifies us that the user has made a selection
    WeekdaySelected(String),

    /// Wraps messages addressed to the autocomplete component
    CountryAutocomplete(autocomplete::Msg),
    /// Autocomplete notifies us that the search contents have changed so we can update the suggestions
    CountryInputChange(String),
    /// Autocomplete notifies us that the user has made a selection
    CountrySelected,
}

fn init(_: Url, _orders: &mut impl Orders<Msg>) -> Model {
    Model {
        search_autocomplete: autocomplete::Model::new(Msg::SearchAutocomplete, |s| Msg::SearchInputChange(s.to_owned()), |_| None, |s| Some(Msg::SearchSubmitted(s.to_owned()))),
        search_previous: TSTSet::new(),
        search: None,

        weekday_autocomplete: autocomplete::Model::new(Msg::WeekdayAutocomplete, |s| Msg::WeekdayInputChange(s.to_owned()), |s: &String| Some(Msg::WeekdaySelected(s.to_owned())), |_| None),
        weekday_search: tst::tstset! { "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"},
        weekday_selected: None,

        country_autocomplete: autocomplete::Model::new(Msg::CountryAutocomplete, |s| Msg::CountryInputChange(s.to_owned()), |_| Some(Msg::CountrySelected), |_| None),
        country_search: CountrySearch::default(),
        country_selected: None,
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SearchInputChange(value) => {
            let suggestions = model.search_previous.prefix_iter(&value);
            model.search_autocomplete.set_suggestions(suggestions.collect());
        }
        Msg::SearchSubmitted(value) => {
            model.search_previous.insert(&value);
            model
                .search_autocomplete
                .set_input_value(value.clone());
            model.search = Some(value);
        }
        Msg::SearchAutocomplete(msg) => autocomplete::update(
            msg,
            &mut model.search_autocomplete,
            orders,
        ),

        Msg::WeekdayInputChange(value) => {
            let suggestions = model.weekday_search.prefix_iter(&value);
            model.weekday_autocomplete.set_suggestions(suggestions.collect());
        }
        Msg::WeekdaySelected(value) => {
            model
                .weekday_autocomplete
                .set_input_value(value.clone());
            model.weekday_selected = Some(value);
        }
        Msg::WeekdayAutocomplete(msg) => autocomplete::update(
            msg,
            &mut model.weekday_autocomplete,
            orders,
        ),

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
                "Search (previous entries will be suggested):",
                // the view for the autocomplete box, adding it into the vdom
                autocomplete::view(&model.search_autocomplete),
            ],
            model.search.as_ref().map(|search| {
                div![
                    h3!["You searched for : ", &search],
                ]
            }),
        ],
        section![
            div![
                "Search for a Weekday:",
                // the view for the autocomplete box, adding it into the vdom
                autocomplete::view(&model.weekday_autocomplete),
            ],
            model.weekday_selected.as_ref().map(|selected_weekday| {
                div![
                    h3![&selected_weekday],
                ]
            }),
        ],
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
