#include <QApplication>
#include <QMainWindow>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QLineEdit>
#include <QTextEdit>
#include <QTableWidget>
#include <QTabWidget>
#include <QMenuBar>
#include <QStatusBar>
#include <QProgressBar>
#include <QTimer>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QNetworkAccessManager>
#include <QNetworkRequest>
#include <QNetworkReply>
#include <QAuthenticator>
#include <QMessageBox>
#include <QFileDialog>
#include <QStandardPaths>
#include <QDir>
#include <QSettings>
#include <QCryptographicHash>
#include <QUuid>
#include <QDateTime>

#include "api_client.h"
#include "dashboard_widget.h"
#include "patients_widget.h"
#include "appointments_widget.h"
#include "billing_widget.h"
#include "licence_dialog.h"

class MainWindow : public QMainWindow {
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr) : QMainWindow(parent) {
        setupUI();
        setupConnections();
        loadSettings();
        
        // Check licence on startup
        checkLicenceStatus();
    }

protected:
    void closeEvent(QCloseEvent *event) override {
        saveSettings();
        event->accept();
    }

private slots:
    void showDashboard() {
        tabWidget->setCurrentIndex(0);
    }
    
    void showPatients() {
        tabWidget->setCurrentIndex(1);
    }
    
    void showAppointments() {
        tabWidget->setCurrentIndex(2);
    }
    
    void showBilling() {
        tabWidget->setCurrentIndex(3);
    }
    
    void showLicenceDialog() {
        LicenceDialog dialog(this);
        if (dialog.exec() == QDialog::Accepted) {
            QString licenceKey = dialog.getLicenceKey();
            if (!licenceKey.isEmpty()) {
                activateLicence(licenceKey);
            }
        }
    }
    
    void refreshData() {
        if (dashboardWidget) {
            dashboardWidget->refreshData();
        }
    }
    
    void logout() {
        settings->setValue("auth/token", "");
        settings->setValue("auth/expires_at", 0);
        statusBar()->showMessage("Logged out", 3000);
        
        // Show login dialog
        // TODO: Implement login dialog
    }
    
    void onLicenceValidationComplete(bool isValid, const QString &message) {
        if (isValid) {
            statusBar()->showMessage("Licence validated successfully", 3000);
            licenceStatusLabel->setText("Valid");
            licenceStatusLabel->setStyleSheet("color: green;");
        } else {
            statusBar()->showMessage("Licence validation failed: " + message, 5000);
            licenceStatusLabel->setText("Invalid");
            licenceStatusLabel->setStyleSheet("color: red;");
        }
    }

private:
    void setupUI() {
        setWindowTitle("Health OS - Hospital Management System");
        setMinimumSize(1200, 800);
        resize(1400, 900);
        
        // Create central widget
        QWidget *centralWidget = new QWidget(this);
        setCentralWidget(centralWidget);
        
        // Create main layout
        QVBoxLayout *mainLayout = new QVBoxLayout(centralWidget);
        
        // Create header with licence status
        createHeader();
        mainLayout->addWidget(headerWidget);
        
        // Create tab widget
        tabWidget = new QTabWidget(this);
        createTabs();
        mainLayout->addWidget(tabWidget);
        
        // Create status bar
        createStatusBar();
        
        // Create menu bar
        createMenuBar();
    }
    
    void createHeader() {
        headerWidget = new QWidget(this);
        QHBoxLayout *headerLayout = new QHBoxLayout(headerWidget);
        
        QLabel *titleLabel = new QLabel("Health OS Hospital Management", this);
        titleLabel->setStyleSheet("font-size: 18px; font-weight: bold; color: #2c3e50;");
        
        licenceStatusLabel = new QLabel("Checking...", this);
        licenceStatusLabel->setStyleSheet("font-size: 12px; color: orange;");
        
        QPushButton *refreshButton = new QPushButton("Refresh", this);
        refreshButton->setMaximumWidth(80);
        connect(refreshButton, &QPushButton::clicked, this, &MainWindow::refreshData);
        
        headerLayout->addWidget(titleLabel);
        headerLayout->addStretch();
        headerLayout->addWidget(licenceStatusLabel);
        headerLayout->addWidget(refreshButton);
        
        headerWidget->setStyleSheet("background-color: #ecf0f1; padding: 10px; border-bottom: 1px solid #ddd;");
    }
    
    void createTabs() {
        // Dashboard Tab
        dashboardWidget = new DashboardWidget(this);
        tabWidget->addTab(dashboardWidget, "Dashboard");
        
        // Patients Tab
        patientsWidget = new PatientsWidget(this);
        tabWidget->addTab(patientsWidget, "Patients");
        
        // Appointments Tab
        appointmentsWidget = new AppointmentsWidget(this);
        tabWidget->addTab(appointmentsWidget, "Appointments");
        
        // Billing Tab
        billingWidget = new BillingWidget(this);
        tabWidget->addTab(billingWidget, "Billing");
        
        // Connect signals
        connect(dashboardWidget, &DashboardWidget::licenceValidationComplete,
                this, &MainWindow::onLicenceValidationComplete);
    }
    
    void createStatusBar() {
        QProgressBar *progressBar = new QProgressBar(this);
        progressBar->setVisible(false);
        progressBar->setMaximumWidth(200);
        
        statusBar()->addPermanentWidget(progressBar);
        statusBar()->showMessage("Ready", 3000);
    }
    
    void createMenuBar() {
        QMenuBar *menuBar = menuBar();
        
        // File Menu
        QMenu *fileMenu = menuBar->addMenu("&File");
        
        QAction *newPatientAction = fileMenu->addAction("&New Patient");
        newPatientAction->setShortcut(QKeySequence("Ctrl+N"));
        connect(newPatientAction, &QAction::triggered, this, [this]() {
            showPatients();
            if (patientsWidget) {
                patientsWidget->showNewPatientDialog();
            }
        });
        
        QAction *newAppointmentAction = fileMenu->addAction("New &Appointment");
        newAppointmentAction->setShortcut(QKeySequence("Ctrl+A"));
        connect(newAppointmentAction, &QAction::triggered, this, [this]() {
            showAppointments();
            if (appointmentsWidget) {
                appointmentsWidget->showNewAppointmentDialog();
            }
        });
        
        fileMenu->addSeparator();
        
        QAction *exitAction = fileMenu->addAction("E&xit");
        exitAction->setShortcut(QKeySequence("Ctrl+Q"));
        connect(exitAction, &QAction::triggered, this, &QWidget::close);
        
        // Tools Menu
        QMenu *toolsMenu = menuBar->addMenu("&Tools");
        
        QAction *refreshAction = toolsMenu->addAction("&Refresh Data");
        refreshAction->setShortcut(QKeySequence("F5"));
        connect(refreshAction, &QAction::triggered, this, &MainWindow::refreshData);
        
        QAction *licenceAction = toolsMenu->addAction("&Licence Management");
        connect(licenceAction, &QAction::triggered, this, &MainWindow::showLicenceDialog);
        
        // Help Menu
        QMenu *helpMenu = menuBar->addMenu("&Help");
        
        QAction *aboutAction = helpMenu->addAction("&About");
        connect(aboutAction, &QAction::triggered, this, [this]() {
            QMessageBox::about(this, "About Health OS",
                "Health OS Hospital Management System\n\n"
                "Version: 1.0.0\n"
                "A comprehensive healthcare management platform\n\n"
                "© 2024 Health OS Team");
        });
    }
    
    void setupConnections() {
        // Timer for periodic licence validation
        licenceTimer = new QTimer(this);
        connect(licenceTimer, &QTimer::timeout, this, &MainWindow::checkLicenceStatus);
        licenceTimer->start(30 * 60 * 1000); // 30 minutes
    }
    
    void checkLicenceStatus() {
        QString licenceKey = settings->value("licence/key", "").toString();
        if (licenceKey.isEmpty()) {
            licenceStatusLabel->setText("Not Configured");
            licenceStatusLabel->setStyleSheet("color: red;");
            return;
        }
        
        // Validate licence
        QString hardwareFingerprint = generateHardwareFingerprint();
        
        QJsonObject requestData;
        requestData["licence_key"] = licenceKey;
        requestData["hardware_fingerprint"] = hardwareFingerprint;
        requestData["client_id"] = QUuid::createUuid().toString(QUuid::WithoutBraces);
        
        // Send validation request
        // This would be implemented in ApiClient
        // For now, simulate validation
        QTimer::singleShot(1000, this, [this]() {
            onLicenceValidationComplete(true, "Licence is valid");
        });
    }
    
    void activateLicence(const QString &licenceKey) {
        QString hardwareFingerprint = generateHardwareFingerprint();
        
        QJsonObject requestData;
        requestData["licence_key"] = licenceKey;
        requestData["hardware_fingerprint"] = hardwareFingerprint;
        requestData["client_id"] = QUuid::createUuid().toString(QUuid::WithoutBraces);
        
        // Send activation request
        // This would be implemented in ApiClient
        // For now, simulate activation
        settings->setValue("licence/key", licenceKey);
        settings->setValue("licence/hardware_fingerprint", hardwareFingerprint);
        settings->setValue("licence/validated_at", QDateTime::currentDateTime().toString(Qt::ISODate));
        
        licenceStatusLabel->setText("Valid");
        licenceStatusLabel->setStyleSheet("color: green;");
        statusBar()->showMessage("Licence activated successfully", 3000);
    }
    
    QString generateHardwareFingerprint() {
        QStringList components;
        
        // System information
        components << "OS:" + QSysInfo::productType();
        components << "VERSION:" + QSysInfo::productVersion();
        components << "ARCH:" + QSysInfo::currentCpuArchitecture();
        components << "HOSTNAME:" + QSysInfo::machineHostName();
        
        // Create SHA-256 hash
        QString fingerprint = components.join("|");
        QByteArray hash = QCryptographicHash::hash(fingerprint.toUtf8(), QCryptographicHash::Sha256);
        
        return hash.toHex();
    }
    
    void loadSettings() {
        settings = new QSettings("HealthOS", "HMS", this);
        
        // Load window geometry
        restoreGeometry(settings->value("geometry").toByteArray());
        restoreState(settings->value("windowState").toByteArray());
    }
    
    void saveSettings() {
        settings->setValue("geometry", saveGeometry());
        settings->setValue("windowState", saveState());
    }

private:
    QWidget *headerWidget;
    QTabWidget *tabWidget;
    DashboardWidget *dashboardWidget;
    PatientsWidget *patientsWidget;
    AppointmentsWidget *appointmentsWidget;
    BillingWidget *billingWidget;
    QLabel *licenceStatusLabel;
    QSettings *settings;
    QTimer *licenceTimer;
};

int main(int argc, char *argv[]) {
    QApplication app(argc, argv);
    
    app.setApplicationName("Health OS HMS");
    app.setApplicationVersion("1.0.0");
    app.setOrganizationName("Health OS");
    app.setOrganizationDomain("healthos.app");
    
    MainWindow window;
    window.show();
    
    return app.exec();
}

#include "main.moc"
