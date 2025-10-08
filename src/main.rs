use agent_runtime::session_creation::session_create::{
    create_session, CreateSessionRequest,
    save_session_to_ram, get_session_from_ram, list_session_ids
};

fn main() {
    let (session, receipt) = create_session(CreateSessionRequest {
        system_prompt: Some("You are an AI assistant.".into()),
        user_prompt_snapshot: Some("Hello".into()),
        max_tokens: Some(2048),
        max_duration_ms: Some(60_000),
        max_context_bytes: Some(256 * 1024),
    });

    save_session_to_ram(&session);

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());
    println!("{:?}", list_session_ids());

    let fetched = get_session_from_ram(&receipt.session_id).unwrap();
    println!("{}", serde_json::to_string_pretty(&*fetched).unwrap());
}
