use agent_runtime::session_creation::session_create::CreateSessionRequest;
use agent_runtime::session_manager::SessionManager;
use agent_runtime::session_inference::infer_once_with_input;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("--- Rust Agent Runtime demo starting ---");

    let (_handle, receipt) = SessionManager::create_session(CreateSessionRequest {
        system_prompt: Some("You are an AI assistant running inside a Rust agent runtime.".into()),
        user_prompt_snapshot: None,
        max_tokens: Some(512),
        max_duration_ms: Some(60_000),
        max_context_bytes: Some(256 * 1024),
    });

    println!("Created session: {}", receipt.session_id);

    SessionManager::set_session_model(&receipt.session_id, "llama3.2-vision:latest")
        .expect("failed to set model");
    println!("Model set for session");

    let _ = SessionManager::start_session(&receipt.session_id)
        .expect("failed to start session");
    println!("Session started");

    let user_input = "what is ai agent?";
    println!("Calling infer_once_with_input...");

    match infer_once_with_input(&receipt.session_id, user_input).await {
        Ok(resp) => {
            println!("LLM RESPONSE:\n{resp}");
        }
        Err(e) => {
            println!("infer_once_with_input error: {e:?}");
        }
    }

    sleep(Duration::from_secs(3)).await;
    println!("--- done ---");
}
