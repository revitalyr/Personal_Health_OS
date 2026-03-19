import React, { useState, useEffect } from 'react';
import { useRouter } from 'next/router';
import Head from 'next/head';
import QRReader from 'react-qrcode-reader';
import axios from 'axios';
import toast, { Toaster } from 'react-hot-toast';

interface PatientData {
  id: string;
  name: string;
  dateOfBirth?: string;
  profiles: UserProfile[];
  aiReport?: AIReport;
  timeline: TimelineEvent[];
  documents: Document[];
}

interface UserProfile {
  id: string;
  name: string;
  relationship: string;
  dateOfBirth?: string;
}

interface AIReport {
  id: string;
  summary: string;
  timeline: string;
  recommendations: string[];
  generatedAt: string;
}

interface TimelineEvent {
  id: string;
  type: 'symptom' | 'medication' | 'visit' | 'diagnosis' | 'lab_result';
  date: string;
  description: string;
  severity?: number;
}

interface Document {
  id: string;
  type: string;
  name: string;
  uploadDate: string;
  url?: string;
}

const DoctorViewer: React.FC = () => {
  const router = useRouter();
  const [patientData, setPatientData] = useState<PatientData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showQRScanner, setShowQRScanner] = useState(false);
  const [accessToken, setAccessToken] = useState<string | null>(null);

  useEffect(() => {
    const { token } = router.query;
    if (token && typeof token === 'string') {
      setAccessToken(token);
      loadPatientData(token);
    }
  }, [router.query]);

  const loadPatientData = async (token: string) => {
    try {
      setLoading(true);
      setError(null);

      const response = await axios.get(`${process.env.NEXT_PUBLIC_API_URL}/doctor/view`, {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      setPatientData(response.data);
    } catch (err: any) {
      const errorMessage = err.response?.data?.message || 'Failed to load patient data';
      setError(errorMessage);
      toast.error(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleQRScan = async (result: string) => {
    try {
      setShowQRScanner(false);
      
      // Extract token from QR code URL
      const tokenMatch = result.match(/\/doctor-view\/(.+)$/);
      if (tokenMatch) {
        const token = tokenMatch[1];
        router.push(`?token=${token}`, undefined, { shallow: true });
      } else {
        toast.error('Invalid QR code format');
      }
    } catch (err) {
      toast.error('Failed to process QR code');
    }
  };

  const handleManualToken = (token: string) => {
    if (token.trim()) {
      router.push(`?token=${token.trim()}`, undefined, { shallow: true });
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'symptom':
        return '🤒';
      case 'medication':
        return '💊';
      case 'visit':
        return '🏥';
      case 'diagnosis':
        return '📋';
      case 'lab_result':
        return '🔬';
      default:
        return '📄';
    }
  };

  const getSeverityColor = (severity?: number) => {
    if (!severity) return 'text-gray-500';
    if (severity <= 3) return 'text-green-500';
    if (severity <= 6) return 'text-yellow-500';
    return 'text-red-500';
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading patient data...</p>
        </div>
      </div>
    );
  }

  if (error || !patientData) {
    return (
      <div className="min-h-screen bg-gray-50">
        <Head>
          <title>Health OS - Doctor Access</title>
        </Head>
        <Toaster />
        
        <div className="container mx-auto px-4 py-8">
          <div className="max-w-md mx-auto bg-white rounded-lg shadow-md p-6">
            <h1 className="text-2xl font-bold text-gray-900 mb-6 text-center">
              Health OS Doctor Access
            </h1>
            
            <div className="space-y-4">
              <div>
                <button
                  onClick={() => setShowQRScanner(true)}
                  className="w-full bg-blue-600 text-white py-3 px-4 rounded-md hover:bg-blue-700 transition-colors"
                >
                  Scan QR Code
                </button>
              </div>
              
              <div className="relative">
                <div className="absolute inset-0 flex items-center">
                  <div className="w-full border-t border-gray-300"></div>
                </div>
                <div className="relative flex justify-center text-sm">
                  <span className="px-2 bg-white text-gray-500">Or</span>
                </div>
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Enter Access Token
                </label>
                <input
                  type="text"
                  placeholder="Enter access token"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  onKeyPress={(e) => {
                    if (e.key === 'Enter') {
                      handleManualToken((e.target as HTMLInputElement).value);
                    }
                  }}
                />
              </div>
            </div>
            
            {showQRScanner && (
              <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div className="bg-white rounded-lg p-4 max-w-sm w-full mx-4">
                  <div className="flex justify-between items-center mb-4">
                    <h3 className="text-lg font-semibold">Scan QR Code</h3>
                    <button
                      onClick={() => setShowQRScanner(false)}
                      className="text-gray-400 hover:text-gray-600"
                    >
                      ✕
                    </button>
                  </div>
                  <QRReader
                    onResult={handleQRScan}
                    onError={(error) => {
                      console.error('QR Scanner Error:', error);
                      toast.error('Failed to scan QR code');
                    }}
                  />
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <Head>
        <title>Health OS - {patientData.name}</title>
      </Head>
      <Toaster />
      
      {/* Header */}
      <header className="bg-white shadow-sm border-b">
        <div className="container mx-auto px-4 py-4">
          <div className="flex justify-between items-center">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">{patientData.name}</h1>
              <p className="text-sm text-gray-600">
                Patient ID: {patientData.id}
                {patientData.dateOfBirth && ` • DOB: ${formatDate(patientData.dateOfBirth)}`}
              </p>
            </div>
            <div className="text-sm text-gray-500">
              Access expires in 15 minutes
            </div>
          </div>
        </div>
      </header>

      <div className="container mx-auto px-4 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2 space-y-8">
            {/* AI Report */}
            {patientData.aiReport && (
              <div className="bg-white rounded-lg shadow-md p-6">
                <h2 className="text-xl font-semibold text-gray-900 mb-4">AI-Generated Report</h2>
                <div className="prose max-w-none">
                  <div className="bg-blue-50 border-l-4 border-blue-400 p-4 mb-4">
                    <h3 className="font-semibold text-blue-900">Summary</h3>
                    <p className="text-blue-800">{patientData.aiReport.summary}</p>
                  </div>
                  
                  <div className="mb-4">
                    <h3 className="font-semibold text-gray-900 mb-2">Timeline</h3>
                    <p className="text-gray-700 whitespace-pre-line">{patientData.aiReport.timeline}</p>
                  </div>
                  
                  {patientData.aiReport.recommendations.length > 0 && (
                    <div>
                      <h3 className="font-semibold text-gray-900 mb-2">Recommendations</h3>
                      <ul className="list-disc list-inside text-gray-700 space-y-1">
                        {patientData.aiReport.recommendations.map((rec, index) => (
                          <li key={index}>{rec}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
                <p className="text-sm text-gray-500 mt-4">
                  Generated on {formatDate(patientData.aiReport.generatedAt)}
                </p>
              </div>
            )}

            {/* Timeline */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <h2 className="text-xl font-semibold text-gray-900 mb-4">Medical Timeline</h2>
              <div className="space-y-4">
                {patientData.timeline.map((event) => (
                  <div key={event.id} className="flex items-start space-x-3 pb-4 border-b last:border-b-0">
                    <div className="text-2xl flex-shrink-0">
                      {getEventIcon(event.type)}
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-1">
                        <span className="font-medium text-gray-900 capitalize">
                          {event.type.replace('_', ' ')}
                        </span>
                        <span className="text-sm text-gray-500">
                          {formatDate(event.date)}
                        </span>
                        {event.severity && (
                          <span className={`text-sm font-medium ${getSeverityColor(event.severity)}`}>
                            Severity: {event.severity}/10
                          </span>
                        )}
                      </div>
                      <p className="text-gray-700">{event.description}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Patient Profiles */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Patient Profiles</h3>
              <div className="space-y-3">
                {patientData.profiles.map((profile) => (
                  <div key={profile.id} className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-gray-900">{profile.name}</p>
                      <p className="text-sm text-gray-500 capitalize">{profile.relationship}</p>
                    </div>
                    {profile.dateOfBirth && (
                      <p className="text-sm text-gray-500">
                        {formatDate(profile.dateOfBirth)}
                      </p>
                    )}
                  </div>
                ))}
              </div>
            </div>

            {/* Recent Documents */}
            <div className="bg-white rounded-lg shadow-md p-6">
              <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Documents</h3>
              <div className="space-y-3">
                {patientData.documents.slice(0, 5).map((doc) => (
                  <div key={doc.id} className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-gray-900">{doc.name}</p>
                      <p className="text-sm text-gray-500 capitalize">{doc.type}</p>
                    </div>
                    <p className="text-sm text-gray-500">
                      {formatDate(doc.uploadDate)}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default DoctorViewer;
