use super::{default_suggestion_view, view, Model};
use seed::prelude::*;
use seed::{style, Attrs, Style};

/// Build up a view of the autocomplete component
pub trait ViewBuilder<'m, Ms: 'static, Suggestion: 'm>: Sized {
    fn borrow_default(&mut self) -> &mut ViewBuilderDefault<'m, Ms, Suggestion>;

    fn into_default(self) -> ViewBuilderDefault<'m, Ms, Suggestion>;

    fn new(model: &'m Model<Ms, Suggestion>) -> ViewBuilderDefault<'m, Ms, Suggestion> {
        let menu_style = style! {
          St::BorderRadius => "3px",
          St::BoxShadow => "0 2px 12px rgba(0, 0, 0, 0.1)",
          St::Background => "rgba(255, 255, 255, 0.9)",
          St::Padding => "2px 0",
          St::FontSize => "90%",
          St::Position => "fixed",
          St::Overflow => "auto",
        //   St::MaxHeight => "50%", // TODO: don't cheat, let it flow to the bottom
        };
        ViewBuilderDefault {
            model,
            input_attrs: Attrs::empty(),
            menu_style,
        }
    }

    /// change the input attributes
    fn with_input_attrs(mut self, input_attrs: Attrs) -> Self {
        self.borrow_default().input_attrs = input_attrs;
        self
    }

    /// add more styles to the menu
    fn add_menu_style(mut self, menu_style: Style) -> Self {
        self.borrow_default().menu_style.merge(menu_style);
        self
    }

    /// set the view function for rendering the suggestions
    fn with_suggestion_view<SuggestionView: Fn(&Suggestion, bool) -> Node<Ms>>(
        self,
        suggestion_view: SuggestionView,
    ) -> ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView> {
        ViewBuilderWithSuggestionView {
            view_builder: self.into_default(),
            suggestion_view,
        }
    }
}

/// Builds a view that uses the default suggestion view function
/// The default view function requires that the Suggestion implements `ToString`
pub struct ViewBuilderDefault<'m, Ms, Suggestion> {
    model: &'m Model<Ms, Suggestion>,
    input_attrs: Attrs,
    menu_style: Style,
}

impl<'m, Ms: 'static, Suggestion> ViewBuilder<'m, Ms, Suggestion>
    for ViewBuilderDefault<'m, Ms, Suggestion>
{
    fn borrow_default(&mut self) -> &mut ViewBuilderDefault<'m, Ms, Suggestion> {
        self
    }

    fn into_default(self) -> ViewBuilderDefault<'m, Ms, Suggestion> {
        self
    }
}

impl<'m, Ms: 'static, Suggestion: ToString> IntoNodes<Ms>
    for ViewBuilderDefault<'m, Ms, Suggestion>
{
    fn into_nodes(self) -> Vec<Node<Ms>> {
        let ViewBuilderDefault {
            model,
            input_attrs,
            menu_style,
        } = self;
        view(&model, default_suggestion_view, input_attrs, menu_style)
    }
}

/// Builds a view that uses a custom suggestion view function
pub struct ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView> {
    view_builder: ViewBuilderDefault<'m, Ms, Suggestion>,
    suggestion_view: SuggestionView,
}

impl<'m, Ms: 'static, Suggestion, SuggestionView> ViewBuilder<'m, Ms, Suggestion>
    for ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView>
{
    fn borrow_default(&mut self) -> &mut ViewBuilderDefault<'m, Ms, Suggestion> {
        &mut self.view_builder
    }

    fn into_default(self) -> ViewBuilderDefault<'m, Ms, Suggestion> {
        self.view_builder
    }
}

impl<'m, Ms: 'static, Suggestion, SuggestionView: Fn(&Suggestion, bool) -> Node<Ms>> IntoNodes<Ms>
    for ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView>
{
    fn into_nodes(self) -> Vec<Node<Ms>> {
        let ViewBuilderWithSuggestionView {
            view_builder:
                ViewBuilderDefault {
                    model,
                    input_attrs,
                    menu_style,
                },
            suggestion_view,
        } = self;

        view(&model, suggestion_view, input_attrs, menu_style)
    }
}
