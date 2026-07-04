// module with some "alternative" cli functions
// generally these are function that can be used to
// try / test some functionalities without booting
// the full app

use std::{sync::Arc, time::Instant};

use llm::LLMProvider;

use crate::crllm::llm_call::call_llm;

/// A simple function to test the llm call
/// Will try a sample  call using the llm provider defined by config
/// In case of error, try to get more info on said error
pub fn test_llm_call_sync(llm: Arc<dyn LLMProvider>) {
    log::info!("Starting llm call");

    let start_time = Instant::now();

    let llm_result = call_llm(
        String::from("This is a test prompt, short reply please"),
        crate::LlmMode::Question,
        llm,
    );

    log::info!("Response: {llm_result}");

    let elapsed = start_time.elapsed().as_secs();

    log::info!("LLM called took {elapsed} seconds");
}
