use std::sync::Arc;

use llm::{LLMProvider, chat::ChatMessage};

use crate::{
    AppState, LlmMode, LlmResponseState,
    fileutils::{dirs::create_dir_structure, errors::FileError, text::read_file_text},
    project::Project,
};

pub fn start_call_llm(app_state: &mut AppState) {
    // copy llm logic so we can use it in thread
    let prompt = app_state.current_prompt.clone();
    let prompt_mode = app_state.current_prompt_mode;
    let llm_provider = Arc::clone(&app_state.llm_provider);

    // Create the prompt, based on the prompt settings
    let complete_prompt = create_complete_prompt(
        prompt.as_str(),
        prompt_mode,
        app_state.current_project.as_ref(),
    );

    // Check if we got the complete prompt, otherwise send error to app
    match complete_prompt {
        Ok(actual_prompt) => {
            // Copy the message channel
            let sender = app_state.sender_channel.clone();

            app_state.llm_response_state = LlmResponseState::Waiting;
            app_state.llm_response_secs = 0;

            std::thread::spawn(move || {
                let output = call_llm(actual_prompt, prompt_mode, llm_provider);
                let _result = sender.send(output);
            });
        }
        Err(e) => app_state.error_msg = Some(format!("{e}")),
    }
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

/// Generate the complete prompt, based on the project settings
#[allow(clippy::uninlined_format_args)]
fn create_complete_prompt(
    prompt: &str,
    prompt_mode: LlmMode,
    project: Option<&Project>,
) -> Result<String, FileError> {
    // So we are concating a bunch of strings.
    // This is going to be inefficient, but I am sticking with
    // a bunch of format!'s for now...
    // TODO? look for a more efficient way
    let mut result = format!(
        "The request is a {} request, so please handle request as described in system prompt.",
        prompt_mode
    );

    if let Some(p) = project {
        // Get description.
        match &p.desc_file {
            Some(df) => {
                let project_desc = read_file_text(df)?;
                result = format!(
                    "{}\n\nThe project description is:\n\n```\n{}\n```",
                    result, project_desc
                );
            }
            None => result = format!("{}\n\nNo project description was included.\n\n", result),
        }

        // Create dir structure
        result = format!(
            "{}\n\nThe dir structure of the project is:\n\n{}\n\n",
            result,
            create_dir_structure(&p.directory)
        );

        // Read active files
        for file in p.file_active_list.iter() {
            let file_content = read_file_text(file)?;
            let file_name = file.to_string_lossy();
            result = format!(
                "{}File: {} has the following contents:\n```\n{}```\n\n",
                result, file_name, file_content
            );
        }
    }

    // Of course we add the prompt itself
    result = format!("{}\nThe user prompt is:\n\n{}", result, prompt);

    Ok(result)
}
