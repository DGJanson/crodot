// File with some llm config
use std::{error, sync::Arc};

use serde::Deserialize;
use serde_json;

use llm::{
    LLMProvider,
    builder::{LLMBackend, LLMBuilder},
    chat::ReasoningEffort,
};

fn default_reasoning() -> bool {
    true
}

fn default_reasoning_effort() -> ReasoningEffort {
    ReasoningEffort::Medium
}

fn deserialize_reasoning_effort<'de, D>(deserializer: D) -> Result<ReasoningEffort, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref().map(|s| s.to_lowercase()) {
        Some(s) if s == "low" => Ok(ReasoningEffort::Low),
        Some(s) if s == "medium" => Ok(ReasoningEffort::Medium),
        Some(s) if s == "high" => Ok(ReasoningEffort::High),
        None => Ok(ReasoningEffort::Medium),
        Some(s) => Err(serde::de::Error::custom(format!(
            "unknown reasoning effort: '{s}', expected 'low', 'medium', or 'high'"
        ))),
    }
}

#[derive(Deserialize)]
pub struct CrodotConfig {
    provider: LlmProvider,
    model: String,
    env_var_key: String,
    base_url: Option<String>,
    system_prompt_file: String, // Location of the system prompt
    api_version: Option<String>,
    deployment_id: Option<String>,
    #[serde(default = "default_reasoning")]
    pub reasoning: bool,
    #[serde(
        default = "default_reasoning_effort",
        deserialize_with = "deserialize_reasoning_effort",
        alias = "reasoningEffort"
    )]
    pub reasoning_effort: ReasoningEffort,
    pub projects: Option<Vec<String>>, // possible list of projects
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum LlmProvider {
    AzureOpenAi,
    OpenRouter,
}

pub fn read_llm_config_file(location: &str) -> Result<CrodotConfig, Box<dyn error::Error>> {
    let file_contents = std::fs::read_to_string(location)?;
    let llm_config = read_llm_config_string(&file_contents)?;
    Ok(llm_config)
}

fn read_llm_config_string(file_contents: &str) -> Result<CrodotConfig, Box<dyn error::Error>> {
    let llm_config: CrodotConfig = serde_json::from_str(file_contents)?;
    Ok(llm_config)
}

/// Generates the llm that we use during our session
/// Pass it a location of a config file with assorted settings
pub fn generate_llm_provider(
    llm_config: CrodotConfig,
) -> Result<Arc<dyn LLMProvider>, Box<dyn error::Error>> {
    // Try to get api key from env var
    let api_key = std::env::var(llm_config.env_var_key)?;

    // init builder and set api key and model
    // use a mutable builder, since it is a bit more flexible
    let mut llm_builder = LLMBuilder::new();
    llm_builder = llm_builder.api_key(api_key);
    llm_builder = llm_builder.model(llm_config.model);

    // Get the system prompt and set in builder
    log::info!("Reading {}", llm_config.system_prompt_file);
    let system_prompt_txt = std::fs::read_to_string(llm_config.system_prompt_file)?;
    llm_builder = llm_builder.system(system_prompt_txt);

    // optional settings
    if let Some(base_url) = llm_config.base_url.as_ref() {
        llm_builder = llm_builder.base_url(base_url);
    }

    if let Some(api_version) = llm_config.api_version {
        llm_builder = llm_builder.api_version(api_version);
    }

    if let Some(deployment_id) = llm_config.deployment_id {
        llm_builder = llm_builder.deployment_id(deployment_id);
    }

    llm_builder = llm_builder.reasoning(llm_config.reasoning);
    llm_builder = llm_builder.reasoning_effort(llm_config.reasoning_effort);

    match llm_config.provider {
        LlmProvider::AzureOpenAi => {
            llm_builder = llm_builder.backend(LLMBackend::AzureOpenAI);
        }
        LlmProvider::OpenRouter => {
            // Fall back to OpenRouter's API endpoint if base_url wasn't manually set in JSON
            if llm_config.base_url.is_none() {
                llm_builder = llm_builder.base_url("https://openrouter.ai/api/v1".to_string());
            }
            llm_builder = llm_builder.backend(LLMBackend::OpenRouter);
        }
    }

    let possible_llm_provider = llm_builder.build();
    match possible_llm_provider {
        Ok(provider) => Ok(Arc::from(provider)),
        Err(e) => {
            log::error!("Could not generate LLM, because:");
            log::error!("{e}");
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_read_config() {
        // Create a temp testfile
        // contents do not matter, it just needs to exist
        // We can refer to in test configs
        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().replace("\\", "/");

        let test_llm_config_correct = format!(
            r#"
            {{
                "provider": "azureopenai",
                "env_var_key": "openai_key",
                "model": "gpt-4",
                "system_prompt_file": "{tmpfile_path}"
            }}
            "#
        );

        let llm_config = read_llm_config_string(&test_llm_config_correct).unwrap();
        assert!(matches!(llm_config.provider, LlmProvider::AzureOpenAi));
        assert!(llm_config.reasoning);
        assert!(matches!(
            llm_config.reasoning_effort,
            ReasoningEffort::Medium
        ));

        tmpfile.close().unwrap();
    }

    #[test]
    fn test_read_config_with_reasoning_effort() {
        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().replace("\\", "/");

        let test_llm_config_high = format!(
            r#"
            {{
                "provider": "azureopenai",
                "env_var_key": "openai_key",
                "model": "gpt-4",
                "system_prompt_file": "{tmpfile_path}",
                "reasoningEffort": "high"
            }}
            "#
        );

        let llm_config = read_llm_config_string(&test_llm_config_high).unwrap();
        assert!(matches!(llm_config.reasoning_effort, ReasoningEffort::High));
        assert!(llm_config.reasoning);

        tmpfile.close().unwrap();
    }

    #[test]
    fn test_read_config_with_reasoning_flag() {
        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().replace("\\", "/");

        let test_llm_config_disabled = format!(
            r#"
            {{
                "provider": "azureopenai",
                "env_var_key": "openai_key",
                "model": "gpt-4",
                "system_prompt_file": "{tmpfile_path}",
                "reasoning": false
            }}
            "#
        );

        let llm_config = read_llm_config_string(&test_llm_config_disabled).unwrap();
        assert!(!llm_config.reasoning);

        tmpfile.close().unwrap();
    }

    #[test]
    #[should_panic]
    fn test_read_incorrect_config() {
        // Create a temp testfile
        // contents do not matter, it just needs to exist
        // We can refer to in test configs
        let tmpfile = NamedTempFile::new().unwrap();
        let tmpfile_path = tmpfile.path().to_str().unwrap().replace("\\", "/");

        let test_llm_config_incorrect = format!(
            r#"
            {{
                "provider": "invalidmodel",
                "env_var_key": "openai_key",
                "model": "gpt-4",
                "system_prompt_file": "{tmpfile_path}"
            }}
            "#
        );

        read_llm_config_string(&test_llm_config_incorrect).unwrap();

        tmpfile.close().unwrap();
    }
}