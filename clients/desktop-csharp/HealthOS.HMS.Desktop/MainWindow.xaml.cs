using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Threading;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using HealthOS.HMS.Core.Services;
using HealthOS.HMS.Core.Models;

namespace HealthOS.HMS.Desktop
{
    public partial class MainWindow : Window
    {
        private readonly IServiceProvider _serviceProvider;
        private readonly ILogger<MainWindow> _logger;
        private readonly IApiClient _apiClient;
        private readonly ILicenceService _licenceService;
        
        private DashboardViewModel _dashboardViewModel;
        private PatientsViewModel _patientsViewModel;
        private AppointmentsViewModel _appointmentsViewModel;
        private BillingViewModel _billingViewModel;
        
        public MainWindow(IServiceProvider serviceProvider)
        {
            _serviceProvider = serviceProvider;
            _logger = serviceProvider.GetRequiredService<ILogger<MainWindow>>();
            _apiClient = serviceProvider.GetRequiredService<IApiClient>();
            _licenceService = serviceProvider.GetRequiredService<ILicenceService>();
            
            InitializeComponent();
            InitializeViewModels();
            SetupEventHandlers();
            
            Loaded += MainWindow_Loaded;
        }
        
        private async void MainWindow_Loaded(object sender, RoutedEventArgs e)
        {
            try
            {
                // Check licence status on startup
                await CheckLicenceStatus();
                
                // Load initial dashboard data
                await _dashboardViewModel.LoadDataAsync();
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error loading main window");
                MessageBox.Show($"Error loading application: {ex.Message}", "Error", 
                    MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
        
        private void InitializeViewModels()
        {
            _dashboardViewModel = new DashboardViewModel(_apiClient, _licenceService);
            _patientsViewModel = new PatientsViewModel(_apiClient);
            _appointmentsViewModel = new AppointmentsViewModel(_apiClient);
            _billingViewModel = new BillingViewModel(_apiClient);
            
            // Set DataContext for tabs
            DashboardTab.DataContext = _dashboardViewModel;
            PatientsTab.DataContext = _patientsViewModel;
            AppointmentsTab.DataContext = _appointmentsViewModel;
            BillingTab.DataContext = _billingViewModel;
        }
        
        private void SetupEventHandlers()
        {
            // Tab selection changed
            MainTabControl.SelectionChanged += MainTabControl_SelectionChanged;
            
            // Window events
            StateChanged += MainWindow_StateChanged;
            
            // Keyboard shortcuts
            InputBindings.Add(new KeyBinding(Key.N, ModifierKeys.Control, NewPatient_Command));
            InputBindings.Add(new KeyBinding(Key.A, ModifierKeys.Control, NewAppointment_Command));
            InputBindings.Add(new KeyBinding(Key.F5, Refresh_Command));
        }
        
        private async void MainTabControl_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (e.AddedItems.Count > 0 && e.AddedItems[0] is TabItem selectedTab)
            {
                try
                {
                    switch (selectedTab.Name)
                    {
                        case "DashboardTab":
                            await _dashboardViewModel.LoadDataAsync();
                            break;
                        case "PatientsTab":
                            await _patientsViewModel.LoadPatientsAsync();
                            break;
                        case "AppointmentsTab":
                            await _appointmentsViewModel.LoadAppointmentsAsync();
                            break;
                        case "BillingTab":
                            await _billingViewModel.LoadInvoicesAsync();
                            break;
                    }
                }
                catch (Exception ex)
                {
                    _logger.LogError(ex, $"Error loading tab: {selectedTab.Name}");
                    await Dispatcher.InvokeAsync(() =>
                    {
                        StatusTextBlock.Text = $"Error loading {selectedTab.Header}: {ex.Message}";
                    });
                }
            }
        }
        
        private async void MainWindow_StateChanged(object sender, EventArgs e)
        {
            if (WindowState ==WindowState.Minimized)
            {
                // Pause background operations when minimized
                _logger.LogInformation("Application minimized - pausing background operations");
            }
            else if (WindowState == WindowState.Normal)
            {
                // Resume background operations when restored
                _logger.LogInformation("Application restored - resuming background operations");
                await RefreshCurrentTab();
            }
        }
        
        private async Task CheckLicenceStatus()
        {
            try
            {
                var licenceStatus = await _licenceService.GetLicenceStatusAsync();
                await Dispatcher.InvokeAsync(() =>
                {
                    UpdateLicenceStatus(licenceStatus);
                });
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error checking licence status");
                await Dispatcher.InvokeAsync(() =>
                {
                    LicenceStatusTextBlock.Text = "Error checking licence";
                    LicenceStatusTextBlock.Foreground = Brushes.Red;
                });
            }
        }
        
        private void UpdateLicenceStatus(LicenceStatus licenceStatus)
        {
            if (licenceStatus.IsValid)
            {
                LicenceStatusTextBlock.Text = $"Valid ({licenceStatus.Tier})";
                LicenceStatusTextBlock.Foreground = Brushes.Green;
                LicenceActivateButton.Visibility = Visibility.Collapsed;
            }
            else
            {
                LicenceStatusTextBlock.Text = "Invalid";
                LicenceStatusTextBlock.Foreground = Brushes.Red;
                LicenceActivateButton.Visibility = Visibility.Visible;
            }
            
            StatusTextBlock.Text = licenceStatus.Message;
        }
        
        private async Task RefreshCurrentTab()
        {
            if (MainTabControl.SelectedItem is TabItem selectedTab)
            {
                switch (selectedTab.Name)
                {
                    case "DashboardTab":
                        await _dashboardViewModel.LoadDataAsync();
                        break;
                    case "PatientsTab":
                        await _patientsViewModel.LoadPatientsAsync();
                        break;
                    case "AppointmentsTab":
                        await _appointmentsViewModel.LoadAppointmentsAsync();
                        break;
                    case "BillingTab":
                        await _billingViewModel.LoadInvoicesAsync();
                        break;
                }
            }
        }
        
        // Command implementations
        private async void NewPatient_Command()
        {
            MainTabControl.SelectedItem = PatientsTab;
            await _patientsViewModel.ShowNewPatientDialogAsync();
        }
        
        private async void NewAppointment_Command()
        {
            MainTabControl.SelectedItem = AppointmentsTab;
            await _appointmentsViewModel.ShowNewAppointmentDialogAsync();
        }
        
        private async void Refresh_Command()
        {
            await RefreshCurrentTab();
        }
        
        private async void LicenceActivate_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                var dialog = new LicenceActivationDialog();
                var result = dialog.ShowDialog();
                
                if (result == true)
                {
                    var licenceKey = dialog.LicenceKey;
                    if (!string.IsNullOrEmpty(licenceKey))
                    {
                        var hardwareFingerprint = await _licenceService.GenerateHardwareFingerprintAsync();
                        var validation = await _apiClient.ValidateLicenceAsync(licenceKey, hardwareFingerprint);
                        
                        if (validation.IsValid)
                        {
                            await _licenceService.SaveLicenceAsync(licenceKey, hardwareFingerprint);
                            UpdateLicenceStatus(new LicenceStatus
                            {
                                IsValid = true,
                                Tier = validation.Tier,
                                Message = "Licence activated successfully"
                            });
                            
                            MessageBox.Show("Licence activated successfully!", "Success", 
                                MessageBoxButton.OK, MessageBoxImage.Information);
                        }
                        else
                        {
                            MessageBox.Show($"Licence activation failed: {validation.Message}", "Error", 
                                MessageBoxButton.OK, MessageBoxImage.Error);
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error activating licence");
                MessageBox.Show($"Error activating licence: {ex.Message}", "Error", 
                    MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
        
        private async void MenuItem_Refresh_Click(object sender, RoutedEventArgs e)
        {
            await RefreshCommand();
        }
        
        private async void MenuItem_Licence_Click(object sender, RoutedEventArgs e)
        {
            await LicenceActivate_Click(sender, e);
        }
        
        private async void MenuItem_Settings_Click(object sender, RoutedEventArgs e)
        {
            try
            {
                var dialog = new SettingsDialog();
                dialog.ShowDialog();
            }
            catch (Exception ex)
            {
                _logger.LogError(ex, "Error opening settings");
                MessageBox.Show($"Error opening settings: {ex.Message}", "Error", 
                    MessageBoxButton.OK, MessageBoxImage.Error);
            }
        }
        
        private void MenuItem_About_Click(object sender, RoutedEventArgs e)
        {
            var aboutText = @"Health OS Hospital Management System
Version: 1.0.0
A comprehensive healthcare management platform

© 2024 Health OS Team
Built with .NET and WPF";

            MessageBox.Show(aboutText, "About Health OS", MessageBoxButton.OK, MessageBoxImage.Information);
        }
        
        private void MenuItem_Exit_Click(object sender, RoutedEventArgs e)
        {
            Application.Current.Shutdown();
        }
    }
}
