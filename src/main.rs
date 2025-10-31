use agent_runtime::session_creation::session_create::CreateSessionRequest;
use agent_runtime::session_manager::SessionManager;
use serde_json;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("runtime booting…");

    let (_s1, r1) = SessionManager::create_session(CreateSessionRequest {
        system_prompt: Some("You are an AI assistant.".into()),
        user_prompt_snapshot: Some("Hello".into()),
        max_tokens: Some(2048),
        max_duration_ms: Some(60_000),
        max_context_bytes: Some(256 * 1024),
    });
    let (_s2, _r2) = SessionManager::create_session(CreateSessionRequest {
        system_prompt: Some("System B".into()),
        user_prompt_snapshot: None,
        max_tokens: None,
        max_duration_ms: None,
        max_context_bytes: None,
    });

    println!("Created: {}", serde_json::to_string_pretty(&r1).unwrap());

    let ticker = tokio::spawn(async {
        loop {
            println!("Session listed = {:?}", SessionManager::list_session_ids());
            tokio::time::sleep(Duration::from_secs(20)).await;
        }
    });

    loop {
        println!("\n=== Session Menu ===");
        let ids = SessionManager::list_session_ids();
        if ids.is_empty() {
            println!("No sessions left. Exiting.");
            break;
        }
        for (i, id) in ids.iter().enumerate() {
            println!("[{}] {}", i, id);
        }
        println!("Type: index to START, 'q' to quit");

        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let input = line.trim();

        if input.eq_ignore_ascii_case("q") {
            println!("Bye!");
            break;
        }

        match input.parse::<usize>() {
            Ok(idx) if idx < ids.len() => {
                let id = ids[idx];
                match SessionManager::start_session(&id) {
                    Ok(receipt) => {
                        println!("Started:\n{}", serde_json::to_string_pretty(&receipt).unwrap());
                        if let Some(handle) = SessionManager::get_session(&id) {
                            let guard = handle.read().unwrap();
                            println!("Now:\n{}", serde_json::to_string_pretty(&*guard).unwrap());
                        }
                    }
                    Err(e) => println!("Start error: {:?}", e),
                }
            }
            _ => println!("Invalid input."),
        }
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl+C received, shutting down…");
        }
        _ = async {} => {}
    }

    ticker.abort();
}
