use std::sync::Arc;

use llm::{LLMProvider, chat::ChatMessage};

use crate::{AppState, LlmMode, LlmResponseState};

pub fn start_call_llm(app_state: &mut AppState) {
    // copy llm logic so we can use it in thread
    let prompt = app_state.current_prompt.clone();
    let prompt_mode = app_state.current_prompt_mode;
    let llm_provider = Arc::clone(&app_state.llm_provider);

    // Copy the message channel
    let sender = app_state.sender_channel.clone();

    app_state.llm_response_state = LlmResponseState::Waiting;
    app_state.llm_response_secs = 0;

    std::thread::spawn(move || {
        let output = call_llm(prompt, prompt_mode, llm_provider);
        let result = sender.send(output);
    });
}

pub fn call_llm(
    prompt: String,
    prompt_mode: LlmMode,
    llm_provider: Arc<dyn LLMProvider>,
) -> String {
    match prompt_mode {
        LlmMode::Mock => prompt,
        _ => {
            let messages = vec![ChatMessage::user().content(prompt).build()];

            // We NEED to use tokio to run "async"
            // We actually block, so there is not proper async, intentionally
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match rt {
                Ok(runtime) => {
                    let response_opt = runtime.block_on(llm_provider.chat(&messages));
                    match response_opt {
                        Ok(response) => {
                            // We receive a ChatResponse
                            response.text().unwrap_or_default()
                        }
                        Err(e) => {
                            format!("ERROR::From Llm Call -> {e:?}")
                        }
                    }
                }
                Err(_) => String::from("ERROR::Could not create runtime"),
            }
        }
    }
}
