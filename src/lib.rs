use seed::prelude::*;
use seed::*;
use web_sys::{Element, HtmlInputElement};
use core::fmt::Display;

/// Model of the autocomplete component, one of these is needed in your model for each autocomplete in the view
pub struct Model<Ms, Suggestion = String> {
    msg_mapper: fn(Msg) -> Ms,
    input_changed: Box<dyn Fn(&str) -> Ms>,
    suggestion_selected: Box<dyn Fn(&Suggestion) -> Option<Ms>>,

    input_ref: ElRef<HtmlInputElement>,
    input_value: String,
    selected: Option<Suggestion>,
    suggestions: Vec<Suggestion>,
    is_open: bool,
    highlighted_index: Option<usize>,
    /// Ignore any blur events. This flag is set when hovering over the suggestions. When the suggestion menu is open, the input box must have focus, a click on a suggestion will cause a blur event on the input, closing the menu, before the click event on the suggestion.
    ignore_blur: bool,
    /// Ignore a focus event. This flag is set if a blur is being ignored, and therefore focius is being brought back to the input box.
    ignore_focus: bool,
}

impl<Ms, Suggestion> Model<Ms, Suggestion> {
    pub fn new(
        msg_mapper: fn(Msg) -> Ms,
        input_changed: impl Fn(&str) -> Ms + 'static,
        suggestion_selected: impl Fn(&Suggestion) -> Option<Ms> + 'static,
    ) -> Self {
        Self {
            msg_mapper,
            input_changed: Box::new(input_changed),
            suggestion_selected: Box::new(suggestion_selected),

            input_ref: Default::default(),
            input_value: Default::default(),
            selected: Default::default(),
            suggestions: Default::default(),
            is_open: Default::default(),
            highlighted_index: Default::default(),
            ignore_blur: Default::default(),
            ignore_focus: Default::default(),
        }
    }

    /// Get the last selected suggestion
    pub fn get_selection(&self) -> Option<&Suggestion> {
        self.selected.as_ref()
    }

    /// Change the suggestions in the suggestion box
    pub fn set_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
    }

    /// set the value in the input box
    pub fn set_input_value(&mut self, value: String) {
        self.input_value = value;
    }
}

#[derive(Clone)]
pub enum Msg {
    InputFocus,
    InputBlur,
    InputKeyDown(web_sys::KeyboardEvent),
    InputClick(web_sys::MouseEvent),
    InputChange(String),
    SuggestionClick(usize),
    SuggestionHover(usize),
    SetIgnoreBlur(bool),
}

pub fn update<Ms: 'static, Suggestion: Display + Clone>(msg: Msg, model: &mut Model<Ms, Suggestion>, orders: &mut impl Orders<Ms>) {
    match msg {
        Msg::InputChange(value) => {
            orders.send_msg((*model.input_changed)(&value));
            model.input_value = value;
        }

        Msg::InputFocus => {
            if model.ignore_focus {
                model.ignore_focus = false;
                // TODO is there scroll handling to do here?
                return;
            }
            // TODO handling for focus causing a scroll which could cause a click to be cancelled
            model.is_open = true;
        }

        Msg::InputBlur => {
            if model.ignore_blur {
                model.ignore_focus = true;
                // TODO is there scroll handling to do here?
                model.input_ref.get().unwrap().focus().unwrap();
                return;
            }
            model.is_open = false;
            model.highlighted_index = None;
        }

        Msg::SetIgnoreBlur(value) => model.ignore_blur = value,

        Msg::InputKeyDown(kb_ev) => {
            match kb_ev.key().as_str() {
                "ArrowDown" => {
                    kb_ev.prevent_default();
                    if model.suggestions.is_empty() {
                        return;
                    }
                    let index = model.highlighted_index.map(|i| i + 1).unwrap_or(0);
                    if index < model.suggestions.len() {
                        model.highlighted_index = Some(index);
                        model.is_open = true;
                    }
                }
                "ArrowUp" => {
                    kb_ev.prevent_default();
                    if model.suggestions.is_empty() {
                        return;
                    }
                    let index = model.highlighted_index.unwrap_or_else(|| model.suggestions.len());
                    if index > 0 {
                        model.highlighted_index = Some(index - 1);
                        model.is_open = true;
                    }
                }
                "Enter" => {
                    // Key code 229 is used for selecting items from character selectors (Pinyin, Kana, etc)
                    if kb_ev.key_code() != 13 {
                        return;
                    }
                    // // In case the user is currently hovering over the menu
                    model.ignore_blur = false;
                    if !model.is_open {
                        // menu is closed so there is no selection to accept -> do nothing
                    } else if let Some(highlighted_index) = model.highlighted_index {
                        // text entered + menu item has been highlighted + enter is hit -> update value to that of selected menu item, close the menu
                        kb_ev.prevent_default();
                        let item = &model.suggestions[highlighted_index];
                        model.is_open = false;
                        model.highlighted_index = None;
                        model.selected = Some(item.clone());
                        (*model.suggestion_selected)(&item).map(|msg| orders.send_msg(msg));
                    } else {
                        model.is_open = false;
                    }
                }
                "Escape" => {
                    // In case the user is currently hovering over the menu
                    model.ignore_blur = false;
                    model.highlighted_index = None;
                    model.is_open = false;
                }
                "Tab" => {
                    // In case the user is currently hovering over the menu
                    model.ignore_blur = false;
                }
                _ => {
                    model.is_open = true;
                }
            }
        }

        Msg::InputClick(_mouse_ev) => {
            let element = model.input_ref.get().unwrap();
            if element
                .owner_document()
                .and_then(|doc| doc.active_element())
                .map(|active_element| active_element == element.into())
                .unwrap_or_default()
            {
                model.is_open = true;
            }
        }

        Msg::SuggestionHover(idx) => {
            model.highlighted_index = Some(idx);
        }

        Msg::SuggestionClick(idx) => {
            let item = &model.suggestions[idx];
            model.selected = Some(item.clone());
            model.ignore_blur = false;
            model.is_open = false;
            model.highlighted_index = None;
            (*model.suggestion_selected)(&item).map(|msg| orders.send_msg(msg));
        }
    }
}

fn get_computed_style_float(
    style_declaration: &web_sys::CssStyleDeclaration,
    key: &str,
) -> Option<f64> {
    fn parse(value: String) -> Option<f64> {
        value.parse().ok()
    }
    style_declaration
        .get_property_value(key)
        .ok()
        .and_then(parse)
}

pub fn view<Ms: 'static, Suggestion: Display>(model: &Model<Ms, Suggestion>) -> Vec<Node<Ms>> {
    let mut menu_style = style! {
      St::BorderRadius => "3px",
      St::BoxShadow => "0 2px 12px rgba(0, 0, 0, 0.1)",
      St::Background => "rgba(255, 255, 255, 0.9)",
      St::Padding => "2px 0",
      St::FontSize => "90%",
      St::Position => "fixed",
      St::Overflow => "auto",
      St::MaxHeight => "50%", // TODO: don't cheat, let it flow to the bottom
    };
    if let Some(node) = model.input_ref.get() {
        let node: Element = node.into();
        let rect = node.get_bounding_client_rect();
        let computed_style = window().get_computed_style(&node).unwrap().unwrap();
        let margin_bottom = get_computed_style_float(&computed_style, "marginBottom").unwrap_or(0.);
        let margin_left = get_computed_style_float(&computed_style, "marginLeft").unwrap_or(0.);
        let margin_right = get_computed_style_float(&computed_style, "marginRight").unwrap_or(0.);
        menu_style.merge(style! {
            St::Left => format!("{}px", rect.left() + margin_left),
            St::Top => format!("{}px", rect.bottom() + margin_bottom),
            St::MinWidth => format!("{}px", rect.width() + margin_left + margin_right),
        });
    }

    nodes![
        input![
            el_ref(&model.input_ref),
            attrs! {
                At::Type => "search",
                At::List => "station-suggestions",
                At::Value => &model.input_value,
            },
            input_ev(Ev::Input, Msg::InputChange),
            // input_ev(Ev::Change, Msg::Change),
            ev(Ev::Focus, |_| Msg::InputFocus),
            input_ev(Ev::Blur, |_| Msg::InputBlur),
            keyboard_ev(Ev::KeyDown, Msg::InputKeyDown),
            mouse_ev(Ev::Click, Msg::InputClick),
        ],
        if model.is_open {
            div![
                menu_style,
                attrs! {
                    At::Id => "station-suggestions",
                },
                model.suggestions.iter().enumerate().map(|(idx, suggestion)| div![
                    style! {
                        St::Background => if Some(idx) == model.highlighted_index { "lightgray" } else { "white" },
                        St::Cursor => "default",
                    },
                    suggestion.to_string(),
                    ev(Ev::MouseEnter, move |_| Msg::SuggestionHover(idx)),
                    ev(Ev::Click, move |_| Msg::SuggestionClick(idx)),
                ]),
                ev(Ev::TouchStart, |_| Msg::SetIgnoreBlur(true)),
                ev(Ev::MouseEnter, |_| Msg::SetIgnoreBlur(true)),
                ev(Ev::MouseLeave, |_| Msg::SetIgnoreBlur(false)),
            ]
        } else {
            empty![]
        },
    ].map_msg(model.msg_mapper)
}
