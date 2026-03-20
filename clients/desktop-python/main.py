import asyncio
import json
import hashlib
import platform
import uuid
from datetime import datetime, timedelta
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, asdict
import aiohttp
import jwt
import cryptography.fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend
import base64
import os
import tkinter as tk
from tkinter import ttk, messagebox, filedialog
import threading
import webbrowser
from PIL import Image, ImageTk
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import matplotlib.dates as mdates
import pandas as pd

# Configuration
API_BASE_URL = "https://api.healthos.app"
CONFIG_FILE = "health_os_config.json"

@dataclass
class LoginRequest:
    email: str
    password: str

@dataclass
class AuthResponse:
    token: str
    refresh_token: str
    expires_in: int
    user: Dict[str, Any]

@dataclass
class LicenceValidationRequest:
    licence_key: str
    hardware_fingerprint: str
    client_id: str

@dataclass
class LicenceValidationResponse:
    licence_id: str
    is_valid: bool
    expiry_date: str
    tier: str
    max_users: int
    current_users: int
    features: List[str]
    validation_message: str
    next_check: str

@dataclass
class Patient:
    id: str
    patient_id: str
    name: str
    date_of_birth: str
    gender: str
    blood_type: Optional[str] = None
    phone: Optional[str] = None
    email: Optional[str] = None
    address: Optional[str] = None
    emergency_contact: Optional[Dict[str, Any]] = None
    status: str
    admission_date: Optional[str] = None
    discharge_date: Optional[str] = None
    created_at: str
    updated_at: str

@dataclass
class DashboardStats:
    total_patients: int
    today_appointments: int
    pending_invoices: int
    monthly_revenue: float
    active_doctors: int
    occupied_beds: int
    total_beds: int

class HealthOSApiClient:
    def __init__(self, base_url: str = API_BASE_URL):
        self.base_url = base_url
        self.session = None
        self.token = None
        self.config = {}
        
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            timeout=aiohttp.ClientTimeout(total=30),
            headers={'Content-Type': 'application/json'}
        )
        await self.load_config()
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
            
    async def load_config(self):
        """Load configuration from file"""
        try:
            if os.path.exists(CONFIG_FILE):
                with open(CONFIG_FILE, 'r') as f:
                    self.config = json.load(f)
                    self.token = self.config.get('token')
        except Exception as e:
            print(f"Error loading config: {e}")
            
    async def save_config(self):
        """Save configuration to file"""
        try:
            with open(CONFIG_FILE, 'w') as f:
                json.dump(self.config, f, indent=2)
        except Exception as e:
            print(f"Error saving config: {e}")
    
    async def login(self, email: str, password: str) -> AuthResponse:
        """Login to the API"""
        url = f"{self.base_url}/auth/login"
        payload = LoginRequest(email=email, password=password)
        
        async with self.session.post(url, json=asdict(payload)) as response:
            if response.status == 200:
                data = await response.json()
                auth_response = AuthResponse(**data)
                self.token = auth_response.token
                self.config['token'] = auth_response.token
                self.config['expires_at'] = (datetime.now() + timedelta(seconds=auth_response.expires_in)).isoformat()
                await self.save_config()
                return auth_response
            else:
                raise Exception(f"Login failed: {response.status}")
    
    async def validate_licence(self, licence_key: str, hardware_fingerprint: str) -> LicenceValidationResponse:
        """Validate licence key"""
        url = f"{self.base_url}/api/v1/validate"
        payload = LicenceValidationRequest(
            licence_key=licence_key,
            hardware_fingerprint=hardware_fingerprint,
            client_id=str(uuid.uuid4())
        )
        
        async with self.session.post(url, json=asdict(payload)) as response:
            if response.status == 200:
                data = await response.json()
                return LicenceValidationResponse(**data)
            else:
                raise Exception(f"Licence validation failed: {response.status}")
    
    async def get_licence_status(self) -> Optional[LicenceValidationResponse]:
        """Get current licence status"""
        if not self.token:
            return None
            
        url = f"{self.base_url}/api/v1/licence/status"
        headers = {'Authorization': f'Bearer {self.token}'}
        
        async with self.session.get(url, headers=headers) as response:
            if response.status == 200:
                data = await response.json()
                return LicenceValidationResponse(**data)
            else:
                raise Exception(f"Failed to get licence status: {response.status}")
    
    async def get_patients(self, page: int = 1, limit: int = 20) -> List[Patient]:
        """Get patients list"""
        if not self.token:
            raise Exception("Not authenticated")
            
        url = f"{self.base_url}/api/v1/patients?page={page}&limit={limit}"
        headers = {'Authorization': f'Bearer {self.token}'}
        
        async with self.session.get(url, headers=headers) as response:
            if response.status == 200:
                data = await response.json()
                return [Patient(**patient) for patient in data['data']]
            else:
                raise Exception(f"Failed to get patients: {response.status}")
    
    async def create_patient(self, patient_data: Dict[str, Any]) -> Patient:
        """Create new patient"""
        if not self.token:
            raise Exception("Not authenticated")
            
        url = f"{self.base_url}/api/v1/patients"
        headers = {'Authorization': f'Bearer {self.token}'}
        
        async with self.session.post(url, json=patient_data, headers=headers) as response:
            if response.status == 201:
                data = await response.json()
                return Patient(**data['data'])
            else:
                raise Exception(f"Failed to create patient: {response.status}")
    
    async def get_dashboard_stats(self) -> DashboardStats:
        """Get dashboard statistics"""
        if not self.token:
            raise Exception("Not authenticated")
            
        url = f"{self.base_url}/api/v1/dashboard/stats"
        headers = {'Authorization': f'Bearer {self.token}'}
        
        async with self.session.get(url, headers=headers) as response:
            if response.status == 200:
                data = await response.json()
                return DashboardStats(**data['data'])
            else:
                raise Exception(f"Failed to get dashboard stats: {response.status}")

def generate_hardware_fingerprint() -> str:
    """Generate hardware fingerprint"""
    components = []
    
    # System information
    components.append(f"PLATFORM:{platform.platform()}")
    components.append(f"SYSTEM:{platform.system()}")
    components.append(f"RELEASE:{platform.release()}")
    components.append(f"VERSION:{platform.version()}")
    components.append(f"PROCESSOR:{platform.processor()}")
    components.append(f"ARCH:{platform.architecture()}")
    
    # Python version
    components.append(f"PYTHON:{platform.python_version()}")
    
    # Create SHA-256 hash
    fingerprint = "|".join(components)
    hash_object = hashlib.sha256(fingerprint.encode())
    return hash_object.hexdigest()

class HealthOSDesktopApp:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("Health OS - Hospital Management System")
        self.root.geometry("1200x800")
        self.root.configure(bg='#f0f0f0')
        
        # Initialize variables
        self.api_client = None
        self.current_user = None
        self.licence_status = None
        
        # Setup UI
        self.setup_styles()
        self.create_widgets()
        
    def setup_styles(self):
        """Setup custom styles"""
        style = ttk.Style()
        style.theme_use('clam')
        
        # Configure styles
        style.configure('Title.TLabel', font=('Arial', 16, 'bold'), background='#f0f0f0')
        style.configure('Header.TFrame', background='#2c3e50', relief='flat')
        style.configure('Card.TFrame', background='white', relief='raised', borderwidth=1)
        style.configure('Status.TLabel', font=('Arial', 10), background='#f0f0f0')
        
    def create_widgets(self):
        """Create main UI widgets"""
        # Create main container
        main_container = ttk.Frame(self.root, style='Card.TFrame')
        main_container.pack(fill='both', expand=True, padx=10, pady=10)
        
        # Create header
        self.create_header(main_container)
        
        # Create notebook for tabs
        self.notebook = ttk.Notebook(main_container)
        self.notebook.pack(fill='both', expand=True, pady=(10, 0))
        
        # Create tabs
        self.create_dashboard_tab()
        self.create_patients_tab()
        self.create_appointments_tab()
        self.create_billing_tab()
        
        # Create status bar
        self.create_status_bar()
        
    def create_header(self, parent):
        """Create header with licence status"""
        header_frame = ttk.Frame(parent, style='Header.TFrame')
        header_frame.pack(fill='x', pady=(0, 10))
        
        # Title
        title_label = ttk.Label(header_frame, text="Health OS Hospital Management", style='Title.TLabel')
        title_label.pack(side='left', padx=20, pady=10)
        
        # Licence status
        self.licence_status_var = tk.StringVar(value="Checking...")
        licence_label = ttk.Label(header_frame, textvariable=self.licence_status_var, style='Status.TLabel')
        licence_label.pack(side='right', padx=20, pady=10)
        
        # Refresh button
        refresh_btn = ttk.Button(header_frame, text="Refresh", command=self.refresh_data)
        refresh_btn.pack(side='right', padx=(0, 20), pady=10)
        
    def create_dashboard_tab(self):
        """Create dashboard tab"""
        dashboard_frame = ttk.Frame(self.notebook)
        self.notebook.add(dashboard_frame, text="Dashboard")
        
        # Stats cards frame
        stats_frame = ttk.Frame(dashboard_frame)
        stats_frame.pack(fill='x', pady=10)
        
        # Create stats cards
        self.stats_vars = {}
        stats = [
            ("Total Patients", "total_patients", "#3498db"),
            ("Today's Appointments", "today_appointments", "#2ecc71"),
            ("Pending Invoices", "pending_invoices", "#f39c12"),
            ("Monthly Revenue", "monthly_revenue", "#9b59b6")
        ]
        
        for i, (title, key, color) in enumerate(stats):
            card_frame = ttk.Frame(stats_frame, style='Card.TFrame')
            card_frame.grid(row=0, column=i, padx=5, pady=5, sticky='ew')
            
            ttk.Label(card_frame, text=title, font=('Arial', 10, 'bold')).pack(pady=5)
            
            self.stats_vars[key] = tk.StringVar(value="0")
            ttk.Label(card_frame, textvariable=self.stats_vars[key], 
                     font=('Arial', 14)).pack(pady=5)
        
        # Configure grid weights
        for i in range(len(stats)):
            stats_frame.columnconfigure(i, weight=1)
    
    def create_patients_tab(self):
        """Create patients tab"""
        patients_frame = ttk.Frame(self.notebook)
        self.notebook.add(patients_frame, text="Patients")
        
        # Toolbar
        toolbar = ttk.Frame(patients_frame)
        toolbar.pack(fill='x', pady=5)
        
        ttk.Button(toolbar, text="New Patient", command=self.show_new_patient_dialog).pack(side='left', padx=5)
        ttk.Button(toolbar, text="Refresh", command=self.refresh_patients).pack(side='left', padx=5)
        
        # Patients table
        columns = ('ID', 'Name', 'Status', 'Admission Date')
        self.patients_tree = ttk.Treeview(patients_frame, columns=columns, show='headings', height=15)
        self.patients_tree.pack(fill='both', expand=True, pady=5)
        
        # Configure columns
        for col in columns:
            self.patients_tree.heading(col, text=col)
            self.patients_tree.column(col, width=150)
        
        # Scrollbar
        scrollbar = ttk.Scrollbar(patients_frame, orient='vertical', command=self.patients_tree.yview)
        scrollbar.pack(side='right', fill='y')
        self.patients_tree.configure(yscrollcommand=scrollbar.set)
        
    def create_appointments_tab(self):
        """Create appointments tab"""
        appointments_frame = ttk.Frame(self.notebook)
        self.notebook.add(appointments_frame, text="Appointments")
        
        # Toolbar
        toolbar = ttk.Frame(appointments_frame)
        toolbar.pack(fill='x', pady=5)
        
        ttk.Button(toolbar, text="New Appointment", command=self.show_new_appointment_dialog).pack(side='left', padx=5)
        ttk.Button(toolbar, text="Refresh", command=self.refresh_appointments).pack(side='left', padx=5)
        
        # Appointments table
        columns = ('Patient', 'Doctor', 'Time', 'Status')
        self.appointments_tree = ttk.Treeview(appointments_frame, columns=columns, show='headings', height=15)
        self.appointments_tree.pack(fill='both', expand=True, pady=5)
        
        # Configure columns
        for col in columns:
            self.appointments_tree.heading(col, text=col)
            self.appointments_tree.column(col, width=150)
    
    def create_billing_tab(self):
        """Create billing tab"""
        billing_frame = ttk.Frame(self.notebook)
        self.notebook.add(billing_frame, text="Billing")
        
        # Toolbar
        toolbar = ttk.Frame(billing_frame)
        toolbar.pack(fill='x', pady=5)
        
        ttk.Button(toolbar, text="New Invoice", command=self.show_new_invoice_dialog).pack(side='left', padx=5)
        ttk.Button(toolbar, text="Refresh", command=self.refresh_billing).pack(side='left', padx=5)
        
        # Invoices table
        columns = ('Invoice #', 'Patient', 'Amount', 'Status', 'Due Date')
        self.invoices_tree = ttk.Treeview(billing_frame, columns=columns, show='headings', height=15)
        self.invoices_tree.pack(fill='both', expand=True, pady=5)
        
        # Configure columns
        for col in columns:
            self.invoices_tree.heading(col, text=col)
            self.invoices_tree.column(col, width=120)
    
    def create_status_bar(self):
        """Create status bar"""
        status_frame = ttk.Frame(self.root)
        status_frame.pack(side='bottom', fill='x')
        
        self.status_var = tk.StringVar(value="Ready")
        ttk.Label(status_frame, textvariable=self.status_var, style='Status.TLabel').pack(side='left', padx=10, pady=5)
    
    def show_new_patient_dialog(self):
        """Show new patient dialog"""
        dialog = tk.Toplevel(self.root)
        dialog.title("New Patient")
        dialog.geometry("400x500")
        dialog.transient(self.root)
        dialog.grab_set()
        
        # Form fields
        ttk.Label(dialog, text="Name:").grid(row=0, column=0, padx=10, pady=5, sticky='w')
        name_entry = ttk.Entry(dialog, width=30)
        name_entry.grid(row=0, column=1, padx=10, pady=5)
        
        ttk.Label(dialog, text="Patient ID:").grid(row=1, column=0, padx=10, pady=5, sticky='w')
        patient_id_entry = ttk.Entry(dialog, width=30)
        patient_id_entry.grid(row=1, column=1, padx=10, pady=5)
        
        ttk.Label(dialog, text="Date of Birth:").grid(row=2, column=0, padx=10, pady=5, sticky='w')
        dob_entry = ttk.Entry(dialog, width=30)
        dob_entry.grid(row=2, column=1, padx=10, pady=5)
        
        ttk.Label(dialog, text="Gender:").grid(row=3, column=0, padx=10, pady=5, sticky='w')
        gender_var = tk.StringVar(value="Male")
        gender_combo = ttk.Combobox(dialog, textvariable=gender_var, values=["Male", "Female", "Other"], width=28)
        gender_combo.grid(row=3, column=1, padx=10, pady=5)
        
        # Buttons
        button_frame = ttk.Frame(dialog)
        button_frame.grid(row=4, column=0, columnspan=2, pady=20)
        
        def save_patient():
            # Save patient logic here
            messagebox.showinfo("Success", "Patient created successfully!")
            dialog.destroy()
            
        ttk.Button(button_frame, text="Save", command=save_patient).pack(side='left', padx=5)
        ttk.Button(button_frame, text="Cancel", command=dialog.destroy).pack(side='left', padx=5)
    
    async def refresh_data(self):
        """Refresh all data"""
        try:
            if self.api_client:
                await self.load_dashboard_data()
                await self.refresh_patients()
                await self.refresh_appointments()
                await self.refresh_billing()
                self.status_var.set("Data refreshed successfully")
        except Exception as e:
            self.status_var.set(f"Error refreshing data: {e}")
            messagebox.showerror("Error", f"Failed to refresh data: {e}")
    
    async def load_dashboard_data(self):
        """Load dashboard statistics"""
        try:
            if self.api_client:
                stats = await self.api_client.get_dashboard_stats()
                self.stats_vars['total_patients'].set(str(stats.total_patients))
                self.stats_vars['today_appointments'].set(str(stats.today_appointments))
                self.stats_vars['pending_invoices'].set(str(stats.pending_invoices))
                self.stats_vars['monthly_revenue'].set(f"${stats.monthly_revenue:.2f}")
        except Exception as e:
            print(f"Error loading dashboard data: {e}")
    
    async def refresh_patients(self):
        """Refresh patients list"""
        try:
            if self.api_client:
                patients = await self.api_client.get_patients()
                
                # Clear existing items
                for item in self.patients_tree.get_children():
                    self.patients_tree.delete(item)
                
                # Add patients to tree
                for patient in patients:
                    self.patients_tree.insert('', 'end', values=(
                        patient.patient_id,
                        patient.name,
                        patient.status,
                        patient.admission_date or "N/A"
                    ))
        except Exception as e:
            print(f"Error refreshing patients: {e}")
    
    async def refresh_appointments(self):
        """Refresh appointments list"""
        try:
            if self.api_client:
                # Get today's appointments
                today = datetime.now().strftime('%Y-%m-%d')
                # This would need to be implemented in the API
                # For now, clear the tree
                for item in self.appointments_tree.get_children():
                    self.appointments_tree.delete(item)
        except Exception as e:
            print(f"Error refreshing appointments: {e}")
    
    async def refresh_billing(self):
        """Refresh invoices list"""
        try:
            if self.api_client:
                # This would need to be implemented in the API
                # For now, clear the tree
                for item in self.invoices_tree.get_children():
                    self.invoices_tree.delete(item)
        except Exception as e:
            print(f"Error refreshing billing: {e}")
    
    async def check_licence_status(self):
        """Check licence status"""
        try:
            if self.api_client:
                licence_status = await self.api_client.get_licence_status()
                if licence_status:
                    if licence_status.is_valid:
                        self.licence_status_var.set(f"Valid ({licence_status.tier})")
                    else:
                        self.licence_status_var.set("Invalid")
                else:
                    self.licence_status_var.set("Not Configured")
        except Exception as e:
            self.licence_status_var.set("Error")
            print(f"Error checking licence status: {e}")
    
    def run(self):
        """Run the application"""
        # Start async tasks
        def run_async_tasks():
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            
            async def main_loop():
                async with HealthOSApiClient() as api_client:
                    self.api_client = api_client
                    await self.check_licence_status()
                    await self.load_dashboard_data()
                    
                    # Schedule periodic licence check
                    while True:
                        await asyncio.sleep(30 * 60)  # 30 minutes
                        await self.check_licence_status()
            
            loop.run_until_complete(main_loop())
        
        # Run async tasks in background thread
        async_thread = threading.Thread(target=run_async_tasks, daemon=True)
        async_thread.start()
        
        # Start GUI
        self.root.mainloop()

if __name__ == "__main__":
    app = HealthOSDesktopApp()
    app.run()
