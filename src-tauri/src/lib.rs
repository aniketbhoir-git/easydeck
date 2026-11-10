pub mod deck;
pub mod error;
pub mod ipc;

pub fn run() {
    let state = ipc::state::AppState::bundled().expect("bundled deck is valid");
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            ipc::commands::load_deck,
            ipc::commands::run_action
        ])
        .run(tauri::generate_context!())
        .expect("error while running EasyDeck");
}
