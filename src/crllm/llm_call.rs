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
        app_state.continue_prompt,
        app_state.last_prompt.as_str(),
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
                            match prompt_mode {
                                // In case of question, we replace spaces with nbsp
                                // For better visualisation
                                LlmMode::Question => {
                                    // response.text().unwrap_or_default().replace(" ", "\u{a0}")
                                    response.text().unwrap_or_default()
                                }
                                // All other cases simple return as is
                                _ => response.text().unwrap_or_default(),
                            }
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
/// To create a prompt based on an earlier prompt, ie continue conversation
/// Set the continue_prompt and last_prompt vars
#[allow(clippy::uninlined_format_args)]
fn create_complete_prompt(
    prompt: &str,
    prompt_mode: LlmMode,
    project: Option<&Project>,
    continue_prompt: bool,
    last_prompt: &str,
) -> Result<String, FileError> {
    // So we are concating a bunch of strings.
    // This is going to be inefficient, but I am sticking with
    // a bunch of format!'s for now...
    // TODO? look for a more efficient way
    let mut result = if continue_prompt {
        last_prompt.to_string()
    } else {
        format!(
            "The request is a {} request, so please handle request as described in system prompt.",
            prompt_mode
        )
    };

    // We only draw the project info once, of course
    if !continue_prompt {
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
            // based on prompt dirs.
            for prompt_dir in p.prompt_dirs.iter() {
                result = format!(
                    "{}\n\nThe dir structure of the {} is:\n\n{}\n\n",
                    result,
                    prompt_dir.description,
                    create_dir_structure(&prompt_dir.path)
                );
            }

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
    }

    // Of course we add the prompt itself
    result = format!("{}\nThe user prompt is:\n\n{}", result, prompt);

    Ok(result)
}
