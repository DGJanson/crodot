// This module contains functions to perform
// general actions on the app state.
// Can be called from view functions /
// event handlers probably

use crate::{AppState, LlmMode, LlmResponseState};

pub fn reset_prompt(state: &mut AppState) {
    state.current_prompt = String::default();
    state.current_prompt_mode = LlmMode::Question;
    state.llm_response = String::default();
    state.llm_response_secs = 0;
    state.llm_response_state = LlmResponseState::Idle;
    state.llm_response_code_response = None;
    state.llm_response_code_selected = 0;
    state.continue_prompt = false;
}

pub fn continue_prompt(state: &mut AppState) {
    state.last_prompt = format!(
        "{}\n\n{}\n\n{}",
        state.last_prompt, state.current_prompt, state.llm_response
    );
    state.continue_prompt = true;
    state.current_prompt = String::default();
    state.llm_response = String::default();
    state.llm_response_secs = 0;
    state.llm_response_state = LlmResponseState::Idle;
    state.llm_response_code_response = None;
    state.llm_response_code_selected = 0;
}
