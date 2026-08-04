mod crllm; // crodot llm crate
mod fileutils;
mod project;
mod state;
mod tui;

mod clitests;

use std::fmt;
use std::sync::{
    Arc,
    mpsc::{Receiver, Sender, channel},
};

use project::Project;

use crate::fileutils::coderesponse::CodingResponse;

enum Screen {
    Title,
    Menu,
    Llm,
    LlmResponse,
    File,
    HandleFile,
}

#[derive(PartialEq, Copy, Clone)]
enum LlmMode {
    Question,
    Coding,
    Mock,
}

impl fmt::Display for LlmMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmMode::Question => write!(f, "Question"),
            LlmMode::Coding => write!(f, "Coding"),
            LlmMode::Mock => write!(f, "Mock"),
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum LlmResponseState {
    Idle,
    Waiting,
    NewResponse,
}

impl fmt::Display for LlmResponseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmResponseState::Idle => write!(f, "LLM: Idle"),
            LlmResponseState::Waiting => write!(f, "LLM: Active"),
            LlmResponseState::NewResponse => write!(f, "LLM: Ready"),
        }
    }
}

struct AppState {
    // UI stuff
    screen: Screen,
    exit: bool,

    // Error Stuff
    error_msg: Option<String>,
    unrecoverable_error: bool,

    // Project stuff
    available_projects: Option<Vec<String>>,
    current_project: Option<Project>,

    // LLM related stuff
    llm_provider: Arc<dyn llm::LLMProvider>,
    current_prompt: String,
    current_prompt_mode: LlmMode,
    llm_response_state: LlmResponseState,
    llm_response: String,
    llm_response_secs: u16,
    llm_response_code_response: Option<CodingResponse>,
    llm_response_code_selected: usize,

    // These vars will allow us to continue prompting
    last_prompt: String,
    continue_prompt: bool,

    // Message channel (for async llm calls)
    sender_channel: Sender<String>,
    receiver_channel: Receiver<String>,

    // Some cross screen communication is handled
    // via this state
    wipe_llm_input_prompt: bool,
}

fn read_command_line_args() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() <= 1 {
        return Err("No config file in arguments".into());
    }

    Ok(args)
}

fn generate_app_state(
    llm_provider: Arc<dyn llm::LLMProvider>,
    projects: Option<Vec<String>>,
) -> AppState {
    // Create the channels for async
    let (tx, rx) = channel::<String>();

    // simply create the app state using the llm_provider
    // use defaults and the above channels for the rest
    AppState {
        screen: Screen::Title,
        exit: false,
        error_msg: None,
        unrecoverable_error: false,
        available_projects: projects,
        current_project: None,
        llm_provider,
        current_prompt: String::default(),
        current_prompt_mode: LlmMode::Question,
        llm_response_state: LlmResponseState::Idle,
        llm_response: String::default(),
        llm_response_secs: 0,
        llm_response_code_response: None,
        llm_response_code_selected: 9999,
        last_prompt: String::default(),
        continue_prompt: false,
        sender_channel: tx,
        receiver_channel: rx,
        wipe_llm_input_prompt: false,
    }
}

fn main() {
    // use a very simple console logger for bootstrapping
    // this will taken over by the tui after it started
    colog::init();

    log::info!("Started application");

    // we need a command line ref to a config file
    let args_opt = read_command_line_args();
    if let Ok(args) = args_opt {
        // Get llm options
        let crodot_config_opt = crllm::llm_config::read_llm_config_file(&args[1]);
        match crodot_config_opt {
            Ok(crodot_config) => {
                // Let us get the projects out first:
                let projects = crodot_config.projects.clone();

                // First we need to generate the llm_provider
                let llm_provider = match crllm::llm_config::generate_llm_provider(crodot_config) {
                    Ok(result) => result,
                    Err(error) => {
                        log::error!("Could not generate LLM provider. Please check config.");
                        log::error!("{error}");
                        return;
                    }
                };

                // Two options now, we look for possible additional arguments
                // If no additional arguments, we simply boot the interface,
                // otherwise we try to handle the arguments
                if args.len() == 2 {
                    // default state, boot the app as intended
                    log::info!("Read config, booting interface");
                    let app_state = generate_app_state(llm_provider, projects);
                    let tui_exit = tui::tuimain::start_tui(app_state);
                    match tui_exit {
                        Ok(_) => log::info!("Exited normally"),
                        Err(err) => {
                            log::error!("{err}");
                            log::error!("Tui errored. Exiting.");
                        }
                    }
                } else {
                    // edge case with additional arguments
                    // These are flags for tests (etc as needed)
                    match args[2].as_ref() {
                        "-llm" => {
                            // In this case we just want to test the llm
                            // We do not create an interface
                            log::info!("Read config, testing LLM call");
                            clitests::test_llm_call_sync(llm_provider);
                        }
                        _ => {
                            log::error!("Unknown cli arg. Exiting");
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("{err}");
                log::error!("Could not read config. Exiting.");
            }
        }
    } else {
        log::error!("Please provide a config file location. Exiting");
    }
}
