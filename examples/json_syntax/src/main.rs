//! AI Agent Builder Example
//!
//! This example demonstrates the exact JSON object syntax shown in the
//! cyrup_sugars README.md file. All syntax works exactly as documented.

use cyrup_sugars::prelude::*;
use sugars_llm::{*};
// Helper trait for the example
trait ExecToText {
    fn exec_to_text(&self) -> String;
}

impl ExecToText for &str {
    fn exec_to_text(&self) -> String {
        format!("Output of: {}", self)
    }
}



fn process_turn() -> String {
    "Processed turn".to_string()
}

#[tokio::main]
#[sugars_macros::json_syntax]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("🤖 AI Agent Builder Example");
    let stream = FluentAi::agent_role("rusty-squire")
    .completion_provider(Mistral::MagistralSmall)
    .temperature(1.0)
    .max_tokens(8000)
    .system_prompt("Act as a Rust developers 'right hand man'.
        You possess deep expertise in using tools to research rust, cargo doc and github libraries.
        You are a patient and thoughtful software artisan; a master of sequential thinking and step-by-step reasoning.
        You excel in compilation triage ...

        ...
        ...

        Today is {{ date }}

        ~ Be Useful, Not Thorough")
    .context( // trait Context
        Context<File>::of("/home/kloudsamurai/ai_docs/mistral_agents.pdf"),
        Context<Files>::glob("/home/kloudsamurai/cyrup-ai/**/*.{md,txt}"),
        Context<Directory>::of("/home/kloudsamurai/cyrup-ai/agent-role/ambient-rust"),
        Context<Github>::glob("/home/kloudsamurai/cyrup-ai/**/*.{rs,md}")
    )
    .mcp_server<Stdio>::bin("/user/local/bin/sweetmcp").init("cargo run -- --stdio")
    .tools( // trait Tool
        Tool<Perplexity>::new({
            "citations" => "true"
        }),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    ) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params({"beta" =>  "true"})
    .memory(Library::named("obsidian_vault"))
    .metadata({ "key" => "val", "foo" => "bar" })
    .on_tool_result(|results| {
        log.info("Agent: Tool results");
    })
    .on_conversation_turn(|conversation, agent| {
        log.info("Agent: " + conversation.last().message())
    })
    .completion_provider(Providers::OpenAI)
    .model(Models::Gpt4OMini)
    .temperature(0.7)
    .on_chunk(|chunk| {
        // Real-time streaming - print each token as it arrives
        // All formatting and coloring happens automatically here
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    })
    .chat(|conversation| {
        let user_input = conversation.latest_user_message();
        
        // Pure logic - no formatting, just conversation flow control
        match user_input.to_lowercase().as_str() {
            "quit" | "exit" | "bye" => {
                ChatLoop::Break
            },
            input if input.starts_with("/help") => {
                ChatLoop::Reprompt("Available commands: /help, quit/exit/bye, or just chat normally!".to_string())
            },
            input if input.contains("code") => {
                let response = format!(
                    "I see you mentioned code! Here's a Rust example: fn main() {{ println!(\"Hello!\"); }} Need help with a specific language?"
                );
                ChatLoop::Reprompt(response)
            },
            _ => {
                // Simple response - builder handles all formatting automatically
                let response = format!(
                    "I understand: '{}'. How can I help you further?", 
                    user_input
                );
                ChatLoop::Reprompt(response)
            }
        }
    })
    .collect();
    Ok(())
}
