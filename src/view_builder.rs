use super::{default_suggestion_view, view, Model};
use seed::prelude::*;
use seed::Attrs;

pub struct ViewBuilder<'m, Ms, Suggestion> {
    model: &'m Model<Ms, Suggestion>,
    attrs: Attrs,
}

pub struct ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView> {
    view_builder: ViewBuilder<'m, Ms, Suggestion>,
    suggestion_view: SuggestionView,
}

impl<'m, Ms, Suggestion> ViewBuilder<'m, Ms, Suggestion> {
    pub fn new(model: &Model<Ms, Suggestion>, attrs: Attrs) -> ViewBuilder<'_, Ms, Suggestion> {
        ViewBuilder { model, attrs }
    }

    pub fn with_suggestion_view<SuggestionView: Fn(&Suggestion, bool) -> Node<Ms>>(
        self,
        suggestion_view: SuggestionView,
    ) -> ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView> {
        ViewBuilderWithSuggestionView {
            view_builder: self,
            suggestion_view: suggestion_view,
        }
    }
}

impl<'m, Ms: 'static, Suggestion: ToString> IntoNodes<Ms> for ViewBuilder<'m, Ms, Suggestion> {
    fn into_nodes(self) -> Vec<Node<Ms>> {
        let ViewBuilder { model, attrs } = self;
        view(&model, default_suggestion_view, attrs)
    }
}

impl<'m, Ms: 'static, Suggestion, SuggestionView: Fn(&Suggestion, bool) -> Node<Ms>> IntoNodes<Ms>
    for ViewBuilderWithSuggestionView<'m, Ms, Suggestion, SuggestionView>
{
    fn into_nodes(self) -> Vec<Node<Ms>> {
        let ViewBuilderWithSuggestionView {
            view_builder: ViewBuilder { model, attrs },
            suggestion_view,
        } = self;
        view(&model, suggestion_view, attrs)
    }
}
