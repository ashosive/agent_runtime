use std::time::Duration;
use agent_runtime::session_creation::session_create::{
    create_session, CreateSessionRequest,
    save_session_to_ram, list_session_ids
};

#[tokio::main]
async fn main() {
    let (s1, _) = create_session(CreateSessionRequest {
        system_prompt: Some("You are an AI assistant.".into()),
        user_prompt_snapshot: Some("Hello".into()),
        max_tokens: Some(2048),
        max_duration_ms: Some(60_000),
        max_context_bytes: Some(256 * 1024),
    });
    save_session_to_ram(&s1);

    let (s2, _) = create_session(CreateSessionRequest {
        system_prompt: Some("System B".into()),
        user_prompt_snapshot: None,
        max_tokens: None,
        max_duration_ms: None,
        max_context_bytes: None,
    });
    save_session_to_ram(&s2);

   let ticker = tokio::spawn(async {
       loop {
           println!("active_sessions = {:?}", list_session_ids());
           tokio::time::sleep(Duration::from_secs(20)).await;
       }
   });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("shutting down");
        }
        _ = wait_until_all_sessions_end() => {
            println!("All sessions ended, shutting down...");
        }
    }


    ticker.abort();
}

async fn wait_until_all_sessions_end() {
    use tokio::time::{sleep, Duration};
    loop {
        if list_session_ids().is_empty() {
            break;
        }
        sleep(Duration::from_secs(2)).await;
    }
}
