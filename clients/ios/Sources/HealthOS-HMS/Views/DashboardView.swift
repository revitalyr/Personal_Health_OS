import Foundation
import SwiftUI
import Charts
import Kingfisher

struct DashboardView: View {
    @StateObject private var viewModel = DashboardViewModel()
    @State private var showingLicenceActivation = false
    @State private var showingNewPatient = false
    @State private var showingNewAppointment = false
    @State private var showingNewInvoice = false
    
    var body: some View {
        NavigationView {
            ScrollView {
                LazyVStack(spacing: 20) {
                    // Header
                    headerSection
                    
                    // Licence Status
                    licenceStatusSection
                    
                    // Stats Cards
                    statsCardsSection
                    
                    // Recent Patients
                    recentPatientsSection
                    
                    // Today's Appointments
                    todayAppointmentsSection
                    
                    // Quick Actions
                    quickActionsSection
                }
                .padding()
            }
            .navigationTitle("Dashboard")
            .navigationBarTitleDisplayMode(.large)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Menu {
                        Button(action: {
                            viewModel.refreshData()
                        }) {
                            Label("Refresh", systemImage: "arrow.clockwise")
                        }
                        
                        Button(action: {
                            viewModel.logout()
                        }) {
                            Label("Logout", systemImage: "arrow.right.square")
                        }
                        
                        Button(action: {
                            // Navigate to settings
                        }) {
                            Label("Settings", systemImage: "gearshape")
                        }
                    } label: {
                        Image(systemName: "ellipsis.circle")
                    }
                }
            }
            .refreshable {
                viewModel.refreshData()
            }
        }
        .sheet(isPresented: $showingLicenceActivation) {
            LicenceActivationView()
                .environmentObject(viewModel)
        }
        .sheet(isPresented: $showingNewPatient) {
            NewPatientView()
                .environmentObject(viewModel)
        }
        .sheet(isPresented: $showingNewAppointment) {
            NewAppointmentView()
                .environmentObject(viewModel)
        }
        .sheet(isPresented: $showingNewInvoice) {
            NewInvoiceView()
                .environmentObject(viewModel)
        }
        .onAppear {
            viewModel.loadDashboardData()
        }
    }
    
    // MARK: - Header Section
    private var headerSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "heart.fill")
                    .font(.largeTitle)
                    .foregroundColor(.red)
                
                VStack(alignment: .leading) {
                    Text("Health OS")
                        .font(.largeTitle)
                        .fontWeight(.bold)
                    
                    Text("Hospital Management System")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
            }
            
            Divider()
        }
    }
    
    // MARK: - Licence Status Section
    private var licenceStatusSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "checkmark.shield.fill")
                    .font(.title2)
                    .foregroundColor(.green)
                
                VStack(alignment: .leading) {
                    Text("Licence Status")
                        .font(.headline)
                        .fontWeight(.semibold)
                    
                    if let licenceStatus = viewModel.licenceStatus {
                        Text(licenceStatus.isValid ? "Valid" : "Invalid")
                            .font(.subheadline)
                            .foregroundColor(licenceStatus.isValid ? .green : .red)
                        
                        Text("Tier: \(licenceStatus.tier)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    } else {
                        Text("Checking...")
                            .font(.subheadline)
                            .foregroundColor(.orange)
                    }
                }
                
                Spacer()
                
                Button(action: {
                    showingLicenceActivation = true
                }) {
                    Text("Activate")
                        .font(.caption)
                }
                .buttonStyle(.bordered)
            }
            .padding()
            .background(Color(.systemGray6))
            .cornerRadius(12)
        }
    }
    
    // MARK: - Stats Cards Section
    private var statsCardsSection: some View {
        LazyVGrid(columns: [
            GridItem(.flexible()),
            GridItem(.flexible())
        ], spacing: 16) {
            StatCardView(
                title: "Total Patients",
                value: "\(viewModel.dashboardStats?.totalPatients ?? 0)",
                icon: "person.3.fill",
                color: .blue,
                trend: viewModel.patientTrend
            )
            
            StatCardView(
                title: "Today's Appointments",
                value: "\(viewModel.dashboardStats?.todayAppointments ?? 0)",
                icon: "calendar.badge.clock",
                color: .green,
                trend: viewModel.appointmentTrend
            )
            
            StatCardView(
                title: "Pending Invoices",
                value: "\(viewModel.dashboardStats?.pendingInvoices ?? 0)",
                icon: "doc.text.fill",
                color: .orange,
                trend: viewModel.invoiceTrend
            )
            
            StatCardView(
                title: "Monthly Revenue",
                value: formatCurrency(viewModel.dashboardStats?.monthlyRevenue ?? 0),
                icon: "dollarsign.circle.fill",
                color: .purple,
                trend: viewModel.revenueTrend
            )
        }
    }
    
    // MARK: - Recent Patients Section
    private var recentPatientsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Recent Patients")
                    .font(.headline)
                    .fontWeight(.semibold)
                
                Spacer()
                
                Button("View All") {
                    // Navigate to patients list
                }
                .font(.caption)
            }
            
            if viewModel.recentPatients.isEmpty {
                EmptyStateView(
                    icon: "person.3",
                    title: "No Recent Patients",
                    subtitle: "New patients will appear here"
                )
                .frame(height: 200)
            } else {
                LazyVStack(spacing: 8) {
                    ForEach(viewModel.recentPatients.prefix(5)) { patient in
                        PatientRowView(patient: patient) {
                            // Navigate to patient details
                            viewModel.navigateToPatientDetails(patient.id)
                        }
                    }
                }
                .padding(.vertical, 8)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
    }
    
    // MARK: - Today's Appointments Section
    private var todayAppointmentsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Today's Appointments")
                    .font(.headline)
                    .fontWeight(.semibold)
                
                Spacer()
                
                Button("View All") {
                    // Navigate to appointments list
                }
                .font(.caption)
            }
            
            if viewModel.todayAppointments.isEmpty {
                EmptyStateView(
                    icon: "calendar.badge.clock",
                    title: "No Appointments Today",
                    subtitle: "Today's schedule is clear"
                )
                .frame(height: 200)
            } else {
                LazyVStack(spacing: 8) {
                    ForEach(viewModel.todayAppointments.prefix(3)) { appointment in
                        AppointmentRowView(appointment: appointment) {
                            // Navigate to appointment details
                            viewModel.navigateToAppointmentDetails(appointment.id)
                        }
                    }
                }
                .padding(.vertical, 8)
            }
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
    }
    
    // MARK: - Quick Actions Section
    private var quickActionsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Actions")
                .font(.headline)
                .fontWeight(.semibold)
            
            LazyVGrid(columns: [
                GridItem(.flexible()),
                GridItem(.flexible()),
                GridItem(.flexible()),
                GridItem(.flexible())
            ], spacing: 16) {
                QuickActionButton(
                    icon: "person.badge.plus",
                    title: "New Patient",
                    color: .blue
                ) {
                    showingNewPatient = true
                }
                
                QuickActionButton(
                    icon: "calendar.badge.plus",
                    title: "New Appointment",
                    color: .green
                ) {
                    showingNewAppointment = true
                }
                
                QuickActionButton(
                    icon: "doc.badge.plus",
                    title: "New Invoice",
                    color: .orange
                ) {
                    showingNewInvoice = true
                }
                
                QuickActionButton(
                    icon: "chart.bar.fill",
                    title: "Reports",
                    color: .purple
                ) {
                    // Navigate to reports
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
    
    // MARK: - Helper Methods
    private func formatCurrency(_ value: Double) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "USD"
        return formatter.string(from: NSNumber(value: value)) ?? "$0.00"
    }
}

// MARK: - Supporting Views
struct StatCardView: View {
    let title: String
    let value: String
    let icon: String
    let color: Color
    let trend: Trend?
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundColor(color)
                
                Spacer()
                
                if let trend = trend {
                    TrendIndicatorView(trend: trend)
                }
            }
            
            Text(value)
                .font(.title)
                .fontWeight(.bold)
            
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding()
        .background(Color(.systemBackground))
        .cornerRadius(12)
        .shadow(color: .black.opacity(0.1), radius: 4, x: 0, y: 2)
    }
}

struct TrendIndicatorView: View {
    let trend: Trend
    
    var body: some View {
        HStack(spacing: 4) {
            Image(systemName: trend.isPositive ? "arrow.up.right" : "arrow.down.right")
                .font(.caption)
                .foregroundColor(trend.isPositive ? .green : .red)
            
            Text("\(trend.percentage, specifier: "%.1f")%")
                .font(.caption)
                .foregroundColor(trend.isPositive ? .green : .red)
        }
    }
}

struct QuickActionButton: View {
    let icon: String
    let title: String
    let color: Color
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundColor(color)
                
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                    .multilineTextAlignment(.center)
            }
            .frame(maxWidth: .infinity)
            .padding()
            .background(Color(.systemBackground))
            .cornerRadius(12)
            .shadow(color: .black.opacity(0.1), radius: 4, x: 0, y: 2)
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct EmptyStateView: View {
    let icon: String
    let title: String
    let subtitle: String
    
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: icon)
                .font(.system(size: 48))
                .foregroundColor(.secondary)
            
            Text(title)
                .font(.headline)
                .fontWeight(.semibold)
                .multilineTextAlignment(.center)
            
            Text(subtitle)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
    }
}

// MARK: - Preview
struct DashboardView_Previews: PreviewProvider {
    static var previews: some View {
        DashboardView()
    }
}
