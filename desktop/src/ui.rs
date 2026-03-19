fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_prevent_default::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            validate_licence,
            check_licence_status,
            get_hardware_fingerprint,
            activate_licence,
            load_patients,
            create_patient,
            load_appointments,
            create_appointment,
            load_invoices,
            create_invoice,
            get_dashboard_stats,
        ])
        .system_tray(create_system_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let window = app.get_window("main").unwrap();
                match id.as_str() {
                    "show" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        window.hide().unwrap();
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    "dashboard" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window.eval("window.location.href = '#/dashboard'").unwrap();
                    }
                    "patients" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window.eval("window.location.href = '#/patients'").unwrap();
                    }
                    "appointments" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window.eval("window.location.href = '#/appointments'").unwrap();
                    }
                    "billing" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window.eval("window.location.href = '#/billing'").unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            
            // Check licence on startup
            let licence_manager = app.state::<AppState>().licence_manager.clone();
            tauri::async_runtime::spawn(async move {
                let manager = licence_manager.lock().unwrap();
                if let Some(licence) = manager.load_saved_licence() {
                    let validation = manager.validate_licence(&licence.licence_key, &licence.hardware_fingerprint).await;
                    match validation {
                        Ok(valid_licence) if valid_licence.is_valid => {
                            manager.save_licence(&licence.licence_key, &licence.hardware_fingerprint).unwrap();
                            manager.start_periodic_validation();
                        }
                        _ => {
                            if let Some(window) = tauri::Window::current() {
                                window.eval("showLicenceActivation()").unwrap();
                            }
                        }
                    }
                } else {
                    if let Some(window) = tauri::Window::current() {
                        window.eval("showLicenceActivation()").unwrap();
                    }
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
