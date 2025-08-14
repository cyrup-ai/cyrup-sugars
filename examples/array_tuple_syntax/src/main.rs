//! AI Agent Builder Example
//!
//! This example demonstrates the exact array tuple syntax shown in the
//! cyrup_sugars README.md file including the elegant on_chunk macro.

use cyrup_sugars::prelude::*;
use sugars_llm::*;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI Agent Builder Example");
    println!("Demonstrating the exact array tuple syntax from README.md");
    println!();

    // Test various array tuple syntax patterns
    println!("✅ Testing array syntax variations:");

    // Single key-value pair
    let _test1 = Tool::<Perplexity>::new([("single", "value")]);
    println!("  - Single pair: Tool::new([('single', 'value')])");

    // Multiple key-value pairs
    let _test2 = Tool::<Perplexity>::new([("key1", "val1"), ("key2", "val2")]);
    println!("  - Multiple pairs: Tool::new([('key1', 'val1'), ('key2', 'val2')])");

    // Zero pairs (empty)
    let _test3 = Tool::<Perplexity>::new([]);
    println!("  - Empty array: Tool::new([])");

    println!("✅ All syntax variations working correctly!");

    let _stream = FluentAi::agent_role("rusty-squire")
    .completion_provider(Mistral::MAGISTRAL_SMALL)
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
    .context(( // trait Context
        Context::<File>::of("/home/kloudsamurai/ai_docs/mistral_agents.pdf"),
        Context::<Files>::glob("/home/kloudsamurai/cyrup-ai/**/*.{md,txt}"),
        Context::<Directory>::of("/home/kloudsamurai/cyrup-ai/agent-role/ambient-rust"),
        Context::<Github>::glob("/home/kloudsamurai/cyrup-ai/**/*.{rs,md}")
    ))
    .mcp_server::<Stdio>().bin("/user/local/bin/sweetmcp").init("cargo run -- --stdio")
    .tools(( // trait Tool
        Tool::<Perplexity>::new([("citations", "true")]),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    )) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params([("beta", "true"), ("foo", "bar")])
    .memory(Library::named("obsidian_vault"))
    .metadata([("key", "val"), ("foo", "bar")])
    .on_tool_result(|_results| {
        // do stuf
    })
    .on_conversation_turn(|_conversation, _agent| {
        println!("[INFO] Agent: Last conversation turn");
        // your custom logic - return a processed message
        process_turn()
    })
    .on_chunk(|result| match result {
        Ok(chunk) => {
            println!("{}", chunk);
            chunk
        },
        Err(error) => {
            // Creates a BadChunk with error information
            ConversationChunk::bad_chunk(error)
        }
    })
    .into_agent() // Agent Now
    .conversation_history(MessageRole::User, "What time is it in Paris, France")
    .conversation_history(MessageRole::System, "The USER is inquiring about the time in Paris, France. Based on their IP address, I see they are currently in Las Vegas, Nevada, USA. The current local time is 16:45")
    .conversation_history(MessageRole::Assistant, "It's 1:45 AM CEST on July 7, 2025, in Paris, France. That's 9 hours ahead of your current time in Las Vegas.")
    .chat("Hello")?; // AsyncStream<MessageChunk

    println!("Chat stream initiated successfully!");

    Ok(())
}
