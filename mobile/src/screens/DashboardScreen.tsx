import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ScrollView,
  Alert,
  RefreshControl,
} from 'react-native';
import { Card, Button, SearchBar, Chip, Badge } from 'react-native-elements';
import Icon from 'react-native-vector-icons/MaterialIcons';
import { useAuth } from '../contexts/AuthContext';
import { API_BASE_URL } from '../config/constants';

interface Patient {
  id: string;
  patient_id: string;
  name: string;
  status: 'active' | 'discharged' | 'transferred';
  admission_date?: string;
  age?: number;
  blood_type?: string;
}

interface Appointment {
  id: string;
  patient_name: string;
  appointment_type: string;
  start_time: string;
  status: string;
  facility_name: string;
}

const DashboardScreen: React.FC = () => {
  const { currentProfile, authToken } = useAuth();
  const [patients, setPatients] = useState<Patient[]>([]);
  const [appointments, setAppointments] = useState<Appointment[]>([]);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  useEffect(() => {
    loadData();
  }, [currentProfile]);

  const loadData = async () => {
    if (!currentProfile || !authToken) return;

    try {
      setLoading(true);
      
      // Load patients
      const patientsResponse = await fetch(`${API_BASE_URL}/patients`, {
        headers: {
          'Authorization': `Bearer ${authToken}`,
        },
      });
      const patientsData = await patientsResponse.json();
      
      // Load appointments
      const appointmentsResponse = await fetch(`${API_BASE_URL}/appointments`, {
        headers: {
          'Authorization': `Bearer ${authToken}`,
        },
      });
      const appointmentsData = await appointmentsResponse.json();

      setPatients(patientsData.data || []);
      setAppointments(appointmentsData.data || []);
    } catch (error) {
      console.error('Error loading dashboard data:', error);
      Alert.alert('Error', 'Failed to load data');
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  const onRefresh = () => {
    setRefreshing(true);
    loadData();
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return '#4CAF50';
      case 'discharged': return '#2196F3';
      case 'transferred': return '#FF9800';
      default: return '#9E9E9E';
    }
  };

  const getAppointmentTypeIcon = (type: string) => {
    switch (type) {
      case 'consultation': return 'stethoscope';
      case 'follow_up': return 'event-repeat';
      case 'procedure': return 'medical-services';
      case 'surgery': return 'local-hospital';
      case 'emergency': return 'warning';
      default: return 'calendar-today';
    }
  };

  const formatDateTime = (dateTime: string) => {
    return new Date(dateTime).toLocaleString();
  };

  const renderPatientCard = (patient: Patient) => (
    <Card key={patient.id} containerStyle={styles.card}>
      <View style={styles.cardHeader}>
        <View style={styles.patientInfo}>
          <Text style={styles.patientName}>{patient.name}</Text>
          <Text style={styles.patientId}>ID: {patient.patient_id}</Text>
        </View>
        <Chip
          title={patient.status}
          buttonStyle={{
            backgroundColor: getStatusColor(patient.status),
          }}
          titleStyle={{ color: 'white' }}
        />
      </View>
      
      <View style={styles.cardContent}>
        {patient.age && (
          <Text style={styles.detailText}>Age: {patient.age}</Text>
        )}
        {patient.blood_type && (
          <Text style={styles.detailText}>Blood Type: {patient.blood_type}</Text>
        )}
        {patient.admission_date && (
          <Text style={styles.detailText}>
            Admitted: {formatDateTime(patient.admission_date)}
          </Text>
        )}
      </View>

      <View style={styles.cardActions}>
        <Button
          title="View Details"
          type="outline"
          buttonStyle={styles.actionButton}
          onPress={() => {
            // Navigate to patient details
          }}
        />
        <Button
          title="Timeline"
          buttonStyle={styles.actionButton}
          onPress={() => {
            // Navigate to patient timeline
          }}
        />
      </View>
    </Card>
  );

  const renderAppointmentCard = (appointment: Appointment) => (
    <Card key={appointment.id} containerStyle={styles.card}>
      <View style={styles.cardHeader}>
        <View style={styles.appointmentInfo}>
          <Text style={styles.appointmentTitle}>
            {appointment.appointment_type.replace('_', ' ').toUpperCase()}
          </Text>
          <Text style={styles.patientName}>{appointment.patient_name}</Text>
        </View>
        <Icon
          name={getAppointmentTypeIcon(appointment.appointment_type)}
          size={24}
          color="#2196F3"
        />
      </View>
      
      <View style={styles.cardContent}>
        <Text style={styles.detailText}>
          <Icon name="schedule" size={16} color="#666" />{' '}
          {formatDateTime(appointment.start_time)}
        </Text>
        <Text style={styles.detailText}>
          <Icon name="location-on" size={16} color="#666" />{' '}
          {appointment.facility_name}
        </Text>
        <Text style={styles.detailText}>
          <Icon name="info" size={16} color="#666" />{' '}
          Status: {appointment.status}
        </Text>
      </View>

      <View style={styles.cardActions}>
        <Button
          title="Check In"
          type="outline"
          buttonStyle={styles.actionButton}
          onPress={() => {
            // Handle check-in
          }}
        />
        <Button
          title="Reschedule"
          buttonStyle={styles.actionButton}
          onPress={() => {
            // Handle reschedule
          }}
        />
      </View>
    </Card>
  );

  return (
    <ScrollView
      style={styles.container}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
      }
    >
      <View style={styles.header}>
        <Text style={styles.welcomeText}>
          Welcome back, {currentProfile?.name || 'Doctor'}!
        </Text>
        <Text style={styles.subtitleText}>
          Hospital Management Dashboard
        </Text>
      </View>

      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>Recent Patients</Text>
          <Badge
            value={patients.length}
            status="primary"
            badgeStyle={{ backgroundColor: '#2196F3' }}
          />
        </View>
        
        {loading ? (
          <Text style={styles.loadingText}>Loading patients...</Text>
        ) : (
          patients.slice(0, 5).map(renderPatientCard)
        )}
        
        {patients.length > 5 && (
          <Button
            title="View All Patients"
            type="outline"
            buttonStyle={styles.viewAllButton}
            onPress={() => {
              // Navigate to all patients
            }}
          />
        )}
      </View>

      <View style={styles.section}>
        <View style={styles.sectionHeader}>
          <Text style={styles.sectionTitle}>Today's Appointments</Text>
          <Badge
            value={appointments.length}
            status="primary"
            badgeStyle={{ backgroundColor: '#4CAF50' }}
          />
        </View>
        
        {loading ? (
          <Text style={styles.loadingText}>Loading appointments...</Text>
        ) : (
          appointments.slice(0, 3).map(renderAppointmentCard)
        )}
        
        {appointments.length > 3 && (
          <Button
            title="View All Appointments"
            type="outline"
            buttonStyle={styles.viewAllButton}
            onPress={() => {
              // Navigate to all appointments
            }}
          />
        )}
      </View>

      <View style={styles.quickActions}>
        <Text style={styles.sectionTitle}>Quick Actions</Text>
        
        <View style={styles.actionGrid}>
          <TouchableOpacity
            style={styles.quickAction}
            onPress={() => {
              // Navigate to patient registration
            }}
          >
            <Icon name="person-add" size={32} color="#2196F3" />
            <Text style={styles.quickActionText}>New Patient</Text>
          </TouchableOpacity>
          
          <TouchableOpacity
            style={styles.quickAction}
            onPress={() => {
              // Navigate to appointment scheduling
            }}
          >
            <Icon name="event" size={32} color="#4CAF50" />
            <Text style={styles.quickActionText}>Schedule</Text>
          </TouchableOpacity>
          
          <TouchableOpacity
            style={styles.quickAction}
            onPress={() => {
              // Navigate to billing
            }}
          >
            <Icon name="receipt" size={32} color="#FF9800" />
            <Text style={styles.quickActionText}>Billing</Text>
          </TouchableOpacity>
          
          <TouchableOpacity
            style={styles.quickAction}
            onPress={() => {
              // Navigate to reports
            }}
          >
            <Icon name="assessment" size={32} color="#9C27B0" />
            <Text style={styles.quickActionText}>Reports</Text>
          </TouchableOpacity>
        </View>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    padding: 20,
    backgroundColor: '#2196F3',
    alignItems: 'center',
  },
  welcomeText: {
    fontSize: 24,
    fontWeight: 'bold',
    color: 'white',
    marginBottom: 5,
  },
  subtitleText: {
    fontSize: 16,
    color: 'rgba(255, 255, 255, 0.8)',
  },
  section: {
    margin: 15,
  },
  sectionHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 15,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#333',
  },
  card: {
    marginBottom: 15,
    borderRadius: 10,
  },
  cardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 10,
  },
  patientInfo: {
    flex: 1,
  },
  patientName: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginBottom: 2,
  },
  patientId: {
    fontSize: 12,
    color: '#666',
  },
  appointmentInfo: {
    flex: 1,
  },
  appointmentTitle: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#333',
    marginBottom: 2,
  },
  cardContent: {
    marginBottom: 15,
  },
  detailText: {
    fontSize: 14,
    color: '#666',
    marginBottom: 5,
  },
  cardActions: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  actionButton: {
    flex: 1,
    marginHorizontal: 5,
  },
  viewAllButton: {
    marginTop: 10,
    backgroundColor: 'transparent',
    borderColor: '#2196F3',
  },
  loadingText: {
    textAlign: 'center',
    color: '#666',
    fontStyle: 'italic',
  },
  quickActions: {
    margin: 15,
  },
  actionGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    justifyContent: 'space-between',
  },
  quickAction: {
    width: '45%',
    backgroundColor: 'white',
    padding: 20,
    borderRadius: 10,
    alignItems: 'center',
    marginBottom: 15,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 3.84,
    elevation: 5,
  },
  quickActionText: {
    marginTop: 8,
    fontSize: 12,
    color: '#333',
    fontWeight: '500',
  },
});

export default DashboardScreen;
