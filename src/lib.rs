use seed::prelude::*;
use seed::*;
use web_sys::{Element, HtmlInputElement};

mod view_builder;
pub use view_builder::{ViewBuilder, ViewBuilderDefault};

#[derive(Debug, Clone)]
pub enum Msg {
    InputFocus,
    InputBlur,
    InputKeyDown(web_sys::KeyboardEvent),
    InputClick(web_sys::MouseEvent),
    InputChange(String),
    SuggestionClick(usize),
    SuggestionHover(usize),
    SetIgnoreSuggestionBlur(bool),
}

/// Model of the autocomplete component, one of these is needed in your model for each autocomplete in the view
pub struct Model<Ms, Suggestion = String> {
    /// Maps the autocomplete message type to the parent message type
    msg_mapper: fn(Msg) -> Ms,

    // Handlers for events that happen in the autocomplete component
    input_changed: Box<dyn Fn(&str) -> Option<Ms>>,
    suggestion_selected: Box<dyn Fn(&Suggestion) -> Option<Ms>>,
    submit: Box<dyn Fn() -> Option<Ms>>,

    input_ref: ElRef<HtmlInputElement>,
    selected: Option<Suggestion>,
    suggestions: Vec<Suggestion>,

    /// Whether the component is open
    is_open: bool,
    /// If an element is highlighted, this referes to its index in the `suggestions` vector
    highlighted_index: Option<usize>,
    /// Ignore any blur events. This flag is set when hovering over the suggestions. When the suggestion menu is open, the input box must have focus, a click on a suggestion will cause a blur event on the input, closing the menu, before the click event on the suggestion.
    ignore_blur: bool,
    /// Ignore a focus event. This flag is set if a blur is being ignored, and therefore focius is being brought back to the input box.
    ignore_focus: bool,
}

impl<Ms: 'static, Suggestion: Clone> Model<Ms, Suggestion> {
    pub fn new(msg_mapper: fn(Msg) -> Ms) -> Self {
        Self {
            msg_mapper,
            input_changed: Box::new(|_| None),
            suggestion_selected: Box::new(|_| None),
            submit: Box::new(|| None),

            input_ref: Default::default(),
            selected: Default::default(),
            suggestions: Default::default(),
            is_open: Default::default(),
            highlighted_index: Default::default(),
            ignore_blur: Default::default(),
            ignore_focus: Default::default(),
        }
    }

    pub fn on_input_change(mut self, input_changed: impl Fn(&str) -> Option<Ms> + 'static) -> Self {
        self.input_changed = Box::new(input_changed);
        self
    }

    pub fn on_selection(
        mut self,
        suggestion_selected: impl Fn(&Suggestion) -> Option<Ms> + 'static,
    ) -> Self {
        self.suggestion_selected = Box::new(suggestion_selected);
        self
    }

    pub fn on_submit(mut self, submit: impl Fn() -> Option<Ms> + 'static) -> Self {
        self.submit = Box::new(submit);
        self
    }

    /// Get the last selected suggestion
    pub fn get_selection(&self) -> Option<&Suggestion> {
        self.selected.as_ref()
    }

    /// Change the suggestions in the suggestion box
    pub fn set_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Ms>) {
        match msg {
            Msg::InputChange(value) => {
                (*self.input_changed)(&value).map(|msg| orders.send_msg(msg));
            }

            Msg::InputFocus => {
                if self.ignore_focus {
                    self.ignore_focus = false;
                    // TODO is there scroll handling to do here?
                    return;
                }
                // TODO handling for focus causing a scroll which could cause a click to be cancelled
                self.is_open = true;
            }

            Msg::InputBlur => {
                if self.ignore_blur {
                    self.ignore_focus = true;
                    // TODO is there scroll handling to do here?
                    self.input_ref.get().unwrap().focus().unwrap();
                    return;
                }
                self.is_open = false;
                self.highlighted_index = None;
            }

            Msg::SetIgnoreSuggestionBlur(value) => self.ignore_blur = value,

            Msg::InputKeyDown(kb_ev) => {
                match kb_ev.key().as_str() {
                    "ArrowDown" => {
                        kb_ev.prevent_default();
                        if self.suggestions.is_empty() {
                            return;
                        }
                        let index = self.highlighted_index.map(|i| i + 1).unwrap_or(0);
                        if index < self.suggestions.len() {
                            self.highlighted_index = Some(index);
                            self.is_open = true;
                        }
                    }
                    "ArrowUp" => {
                        kb_ev.prevent_default();
                        if self.suggestions.is_empty() {
                            return;
                        }
                        let index = self
                            .highlighted_index
                            .unwrap_or_else(|| self.suggestions.len());
                        if index > 0 {
                            self.highlighted_index = Some(index - 1);
                            self.is_open = true;
                        }
                    }
                    "Enter" => {
                        // Key code 229 is used for selecting items from character selectors (Pinyin, Kana, etc)
                        if kb_ev.key_code() != 13 {
                            return;
                        }
                        // // In case the user is currently hovering over the menu
                        self.ignore_blur = false;
                        if !self.is_open {
                            // menu is closed so there is no selection to accept -> do nothing
                            (*self.submit)().map(|msg| orders.send_msg(msg));
                        } else if let Some(highlighted_index) = self.highlighted_index {
                            // text entered + menu item has been highlighted + enter is hit -> update value to that of selected menu item, close the menu
                            kb_ev.prevent_default();
                            let item = &self.suggestions[highlighted_index];
                            self.is_open = false;
                            self.highlighted_index = None;
                            self.selected = Some(item.clone());
                            (*self.suggestion_selected)(&item).map(|msg| orders.send_msg(msg));
                            (*self.submit)().map(|msg| orders.send_msg(msg));
                        } else {
                            self.is_open = false;
                            (*self.submit)().map(|msg| orders.send_msg(msg));
                        }
                    }
                    "Escape" => {
                        // In case the user is currently hovering over the menu
                        self.ignore_blur = false;
                        self.highlighted_index = None;
                        self.is_open = false;
                    }
                    "Tab" => {
                        // In case the user is currently hovering over the menu
                        self.ignore_blur = false;
                    }
                    _ => {
                        self.is_open = true;
                    }
                }
            }

            Msg::InputClick(_mouse_ev) => {
                let element = self.input_ref.get().unwrap();
                if element
                    .owner_document()
                    .and_then(|doc| doc.active_element())
                    .map(|active_element| active_element == element.into())
                    .unwrap_or_default()
                {
                    self.is_open = true;
                }
            }

            Msg::SuggestionHover(idx) => {
                self.highlighted_index = Some(idx);
            }

            Msg::SuggestionClick(idx) => {
                let item = &self.suggestions[idx];
                self.selected = Some(item.clone());
                self.ignore_blur = false;
                self.is_open = false;
                self.highlighted_index = None;
                (*self.suggestion_selected)(&item).map(|msg| orders.send_msg(msg));
                (*self.submit)().map(|msg| orders.send_msg(msg));
            }
        }
    }

    /// Create a `ViewBuilder` to start building a view of the autocomplete component
    pub fn view(&self) -> ViewBuilderDefault<'_, Ms, Suggestion> {
        ViewBuilderDefault::new(self)
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

fn view<Ms: 'static, Suggestion>(
    model: &Model<Ms, Suggestion>,
    suggestion_view: impl Fn(&Suggestion, bool) -> Node<Ms>,
    input_attrs: Attrs,
    mut menu_style: Style,
) -> Vec<Node<Ms>> {
    // if let Some(node) = model.input_ref.get() {
    //     let node: Element = node.into();
    //     let rect = node.get_bounding_client_rect();
    //     let computed_style = window().get_computed_style(&node).unwrap().unwrap();
    //     let margin_bottom = get_computed_style_float(&computed_style, "marginBottom").unwrap_or(0.);
    //     let margin_left = get_computed_style_float(&computed_style, "marginLeft").unwrap_or(0.);
    //     let margin_right = get_computed_style_float(&computed_style, "marginRight").unwrap_or(0.);
    //     menu_style.merge(style! {
    //         St::Left => format!("{}px", rect.left() + margin_left),
    //         St::Top => format!("{}px", rect.bottom() + margin_bottom),
    //         St::MinWidth => format!("{}px", rect.width() + margin_left + margin_right),
    //     });
    // }

    let msg_mapper = model.msg_mapper;

    nodes![div![
        style! {
            St::Display => "inline-block",
            St::Position => "relative",
        },
        input![
            el_ref(&model.input_ref),
            input_attrs,
            input_ev(Ev::Input, Msg::InputChange),
            // input_ev(Ev::Change, Msg::Change),
            simple_ev(Ev::Focus, Msg::InputFocus),
            simple_ev(Ev::Blur, Msg::InputBlur),
            keyboard_ev(Ev::KeyDown, Msg::InputKeyDown),
            mouse_ev(Ev::Click, Msg::InputClick),
        ]
        .map_msg(msg_mapper),
        if model.is_open {
            div![
                menu_style,
                model
                    .suggestions
                    .iter()
                    .enumerate()
                    .map(|(idx, suggestion)| {
                        let mut suggestion_node =
                            suggestion_view(suggestion, Some(idx) == model.highlighted_index);
                        suggestion_node
                            .add_event_handler(
                                simple_ev(Ev::MouseEnter, Msg::SuggestionHover(idx))
                                    .map_msg(msg_mapper),
                            )
                            .add_event_handler(
                                simple_ev(Ev::Click, Msg::SuggestionClick(idx)).map_msg(msg_mapper),
                            );
                        suggestion_node
                    })
                    .collect::<Vec<_>>(),
                ev(Ev::TouchStart, |_| Msg::SetIgnoreSuggestionBlur(true)).map_msg(msg_mapper),
                ev(Ev::MouseEnter, |_| Msg::SetIgnoreSuggestionBlur(true)).map_msg(msg_mapper),
                ev(Ev::MouseLeave, |_| Msg::SetIgnoreSuggestionBlur(false)).map_msg(msg_mapper),
            ]
        } else {
            empty![]
        },
    ]]
}

pub fn default_suggestion_view<Suggestion: ToString, Ms>(
    suggestion: &Suggestion,
    is_highlighted: bool,
) -> Node<Ms> {
    div![
        style! {
            St::Background => if is_highlighted { "lightgray" } else { "white" },
            St::Cursor => "default",
        },
        suggestion.to_string(),
    ]
}
