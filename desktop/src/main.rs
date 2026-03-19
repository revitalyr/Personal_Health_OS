// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, Manager, Menu, MenuEntry, MenuItem, Submenu, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowBuilder,
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey};
use base64::{Engine as _, engine::general_purpose};

mod licence;
mod api;
mod ui;
use licence::{LicenceManager, LicenceValidation};

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    licence_manager: Arc<Mutex<LicenceManager>>,
    api_client: Arc<Mutex<api::ApiClient>>,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
}

#[tauri::command]
async fn validate_licence(
    licence_key: String,
    hardware_fingerprint: String,
    state: tauri::State<'_, AppState>,
) -> Result<LicenceValidation, String> {
    let licence_manager = state.licence_manager.lock().unwrap();
    licence_manager.validate_licence(&licence_key, &hardware_fingerprint)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_licence_status(
    state: tauri::State<'_, AppState>,
) -> Result<Option<LicenceValidation>, String> {
    let licence_manager = state.licence_manager.lock().unwrap();
    Ok(licence_manager.get_current_licence())
}

#[tauri::command]
async fn get_hardware_fingerprint() -> Result<String, String> {
    licence::generate_hardware_fingerprint().map_err(|e| e.to_string())
}

#[tauri::command]
async fn activate_licence(
    licence_key: String,
    state: tauri::State<'_, AppState>,
) -> Result<LicenceValidation, String> {
    let hardware_fingerprint = licence::generate_hardware_fingerprint()
        .map_err(|e| e.to_string())?;
    
    let licence_manager = state.licence_manager.lock().unwrap();
    let validation = licence_manager.validate_licence(&licence_key, &hardware_fingerprint)
        .await
        .map_err(|e| e.to_string())?;
    
    if validation.is_valid {
        licence_manager.save_licence(&licence_key, &hardware_fingerprint)
            .map_err(|e| e.to_string())?;
        
        // Start periodic validation
        licence_manager.start_periodic_validation();
    }
    
    Ok(validation)
}

#[tauri::command]
async fn load_patients(
    page: u32,
    limit: u32,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.get_patients(page, limit).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_patient(
    patient_data: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.create_patient(patient_data).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_appointments(
    date: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.get_appointments(&date).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_appointment(
    appointment_data: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.create_appointment(appointment_data).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_invoices(
    page: u32,
    limit: u32,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.get_invoices(page, limit).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_invoice(
    invoice_data: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.create_invoice(invoice_data).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_dashboard_stats(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api_client = state.api_client.lock().unwrap();
    api_client.get_dashboard_stats().await
        .map_err(|e| e.to_string())
}

fn create_system_tray() -> SystemTray {
    let show = CustomMenuItem::new("show".to_string(), "Show").unwrap();
    let hide = CustomMenuItem::new("hide".to_string(), "Hide").unwrap();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit").unwrap();
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new(
            "dashboard".to_string(),
            "Dashboard",
        ).unwrap())
        .add_item(CustomMenuItem::new(
            "patients".to_string(),
            "Patients",
        ).unwrap())
        .add_item(CustomMenuItem::new(
            "appointments".to_string(),
            "Appointments",
        ).unwrap())
        .add_item(CustomMenuItem::new(
            "billing".to_string(),
            "Billing",
        ).unwrap())
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    
    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    // Initialize licence manager
    let licence_manager = Arc::new(Mutex::new(
        LicenceManager::new().expect("Failed to initialize licence manager")
    ));
    
    // Initialize API client
    let api_client = Arc::new(Mutex::new(
        api::ApiClient::new("http://localhost:8080")
    ));
    
    let app_state = AppState {
        licence_manager,
        api_client,
    };

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
                        // Navigate to dashboard
                        window.eval("window.location.href = '#/dashboard'").unwrap();
                    }
                    "patients" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        // Navigate to patients
                        window.eval("window.location.href = '#/patients'").unwrap();
                    }
                    "appointments" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        // Navigate to appointments
                        window.eval("window.location.href = '#/appointments'").unwrap();
                    }
                    "billing" => {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        // Navigate to billing
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
                            // Show licence activation dialog
                            if let Some(window) = tauri::Window::current() {
                                window.eval("showLicenceActivation()").unwrap();
                            }
                        }
                    }
                } else {
                    // Show licence activation dialog
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
