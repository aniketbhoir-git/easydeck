pub mod error;
pub mod ipc;
pub mod sources;
pub mod volume;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(sources::SourceRunner::default())
        .manage(volume::VolumeService::system())
        .setup(|app| {
            let handle = app.handle().clone();
            // The approval ledger lives in the app data folder (SPEC §4.3), never in a deck.
            // Without a data folder the ledger stays in memory, and the starter deck is
            // approved for this run only.
            let dir = app.path().app_data_dir().ok();
            app.manage(ipc::state::AppState::open(dir.as_deref()).expect("bundled deck is valid"));
            let state = app.state::<ipc::state::AppState>();
            app.state::<sources::SourceRunner>()
                .restart(&handle, state.deck(), |s| state.source_allowed(s));
            app.state::<volume::VolumeService>()
                .start_poller(handle, state.deck());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::load_deck,
            ipc::commands::run_action,
            ipc::commands::validate_deck,
            ipc::commands::risk_review,
            ipc::commands::approve_risk_items,
            ipc::commands::revoke_risk_items,
            volume::volume_get,
            volume::volume_set,
            volume::volume_mute
        ])
        .build(tauri::generate_context!())
        .expect("error while building EasyDeck")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<sources::SourceRunner>().stop();
                app.state::<volume::VolumeService>().stop();
            }
        });
}
