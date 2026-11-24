pub mod error;
pub mod ipc;
pub mod sources;

use tauri::Manager;

pub fn run() {
    let state = ipc::state::AppState::bundled().expect("bundled deck is valid");
    tauri::Builder::default()
        .manage(state)
        .manage(sources::SourceRunner::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<ipc::state::AppState>();
            app.state::<sources::SourceRunner>()
                .restart(&handle, state.deck());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::load_deck,
            ipc::commands::run_action
        ])
        .build(tauri::generate_context!())
        .expect("error while building EasyDeck")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<sources::SourceRunner>().stop();
            }
        });
}
