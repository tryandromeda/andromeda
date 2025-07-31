// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use console::Style;
use reedline::{Prompt, PromptHistorySearch, PromptHistorySearchStatus};

#[derive(Clone)]
pub struct ReplPrompt {
    evaluation_count: usize,
}

impl ReplPrompt {
    pub fn new(count: usize) -> Self {
        Self {
            evaluation_count: count,
        }
    }
}

impl Prompt for ReplPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        let count_style = Style::new().dim();
        format!(
            "{} > ",
            count_style.apply_to(format!("{}", self.evaluation_count))
        )
        .into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_indicator(
        &self,
        _prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        let style = Style::new().dim();
        format!("{}", style.apply_to("...")).into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        format!("({}reverse-search: {}) ", prefix, history_search.term).into()
    }
}
