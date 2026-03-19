import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  Alert,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
} from 'react-native';
import { useAuth } from '../contexts/AuthContext';

const LoginScreen: React.FC = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [phone, setPhone] = useState('');
  const [otp, setOtp] = useState('');
  const [showOTP, setShowOTP] = useState(false);
  const [loginMode, setLoginMode] = useState<'email' | 'phone'>('email');
  
  const { 
    loginEmail, 
    loginGoogle, 
    loginApple, 
    sendOTP, 
    verifyOTP, 
    loading, 
    error, 
    clearError 
  } = useAuth();

  React.useEffect(() => {
    if (error) {
      Alert.alert('Error', error);
      clearError();
    }
  }, [error, clearError]);

  const handleEmailLogin = async () => {
    if (!email || !password) {
      Alert.alert('Error', 'Please fill in all fields');
      return;
    }
    await loginEmail(email, password);
  };

  const handleSendOTP = async () => {
    if (!phone) {
      Alert.alert('Error', 'Please enter phone number');
      return;
    }
    await sendOTP(phone);
    setShowOTP(true);
  };

  const handleVerifyOTP = async () => {
    if (!otp) {
      Alert.alert('Error', 'Please enter OTP code');
      return;
    }
    await verifyOTP(phone, otp);
  };

  const handleGoogleLogin = async () => {
    await loginGoogle();
  };

  const handleAppleLogin = async () => {
    await loginApple();
  };

  return (
    <KeyboardAvoidingView 
      style={styles.container} 
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
    >
      <ScrollView contentContainerStyle={styles.scrollContainer}>
        <View style={styles.header}>
          <Text style={styles.title}>Health OS</Text>
          <Text style={styles.subtitle}>Your Personal Health Assistant</Text>
        </View>

        <View style={styles.form}>
          <View style={styles.modeSelector}>
            <TouchableOpacity
              style={[
                styles.modeButton,
                loginMode === 'email' && styles.modeButtonActive
              ]}
              onPress={() => setLoginMode('email')}
            >
              <Text style={[
                styles.modeButtonText,
                loginMode === 'email' && styles.modeButtonTextActive
              ]}>
                Email
              </Text>
            </TouchableOpacity>
            <TouchableOpacity
              style={[
                styles.modeButton,
                loginMode === 'phone' && styles.modeButtonActive
              ]}
              onPress={() => setLoginMode('phone')}
            >
              <Text style={[
                styles.modeButtonText,
                loginMode === 'phone' && styles.modeButtonTextActive
              ]}>
                Phone
              </Text>
            </TouchableOpacity>
          </View>

          {loginMode === 'email' ? (
            <>
              <TextInput
                style={styles.input}
                placeholder="Email"
                value={email}
                onChangeText={setEmail}
                keyboardType="email-address"
                autoCapitalize="none"
                autoCorrect={false}
              />
              <TextInput
                style={styles.input}
                placeholder="Password"
                value={password}
                onChangeText={setPassword}
                secureTextEntry
              />
              <TouchableOpacity
                style={[styles.button, styles.primaryButton, loading && styles.buttonDisabled]}
                onPress={handleEmailLogin}
                disabled={loading}
              >
                <Text style={styles.buttonText}>
                  {loading ? 'Signing In...' : 'Sign In'}
                </Text>
              </TouchableOpacity>
            </>
          ) : (
            <>
              <TextInput
                style={styles.input}
                placeholder="Phone Number"
                value={phone}
                onChangeText={setPhone}
                keyboardType="phone-pad"
              />
              {!showOTP ? (
                <TouchableOpacity
                  style={[styles.button, styles.primaryButton, loading && styles.buttonDisabled]}
                  onPress={handleSendOTP}
                  disabled={loading}
                >
                  <Text style={styles.buttonText}>
                    {loading ? 'Sending...' : 'Send OTP'}
                  </Text>
                </TouchableOpacity>
              ) : (
                <>
                  <TextInput
                    style={styles.input}
                    placeholder="Enter OTP Code"
                    value={otp}
                    onChangeText={setOtp}
                    keyboardType="number-pad"
                    maxLength={6}
                  />
                  <TouchableOpacity
                    style={[styles.button, styles.primaryButton, loading && styles.buttonDisabled]}
                    onPress={handleVerifyOTP}
                    disabled={loading}
                  >
                    <Text style={styles.buttonText}>
                      {loading ? 'Verifying...' : 'Verify OTP'}
                    </Text>
                  </TouchableOpacity>
                </>
              )}
            </>
          )}

          <View style={styles.divider}>
            <View style={styles.dividerLine} />
            <Text style={styles.dividerText}>OR</Text>
            <View style={styles.dividerLine} />
          </View>

          <TouchableOpacity
            style={[styles.button, styles.googleButton, loading && styles.buttonDisabled]}
            onPress={handleGoogleLogin}
            disabled={loading}
          >
            <Text style={styles.buttonText}>Continue with Google</Text>
          </TouchableOpacity>

          {Platform.OS === 'ios' && (
            <TouchableOpacity
              style={[styles.button, styles.appleButton, loading && styles.buttonDisabled]}
              onPress={handleAppleLogin}
              disabled={loading}
            >
              <Text style={styles.buttonText}>Continue with Apple</Text>
            </TouchableOpacity>
          )}
        </View>

        <View style={styles.footer}>
          <Text style={styles.footerText}>
            Don't have an account? Sign up functionality coming soon!
          </Text>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8f9fa',
  },
  scrollContainer: {
    flexGrow: 1,
    justifyContent: 'center',
    padding: 20,
  },
  header: {
    alignItems: 'center',
    marginBottom: 40,
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#2c3e50',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 16,
    color: '#7f8c8d',
  },
  form: {
    backgroundColor: 'white',
    borderRadius: 12,
    padding: 24,
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 2,
    },
    shadowOpacity: 0.1,
    shadowRadius: 3.84,
    elevation: 5,
  },
  modeSelector: {
    flexDirection: 'row',
    marginBottom: 24,
    backgroundColor: '#f1f3f4',
    borderRadius: 8,
    padding: 4,
  },
  modeButton: {
    flex: 1,
    paddingVertical: 12,
    borderRadius: 6,
    alignItems: 'center',
  },
  modeButtonActive: {
    backgroundColor: 'white',
    shadowColor: '#000',
    shadowOffset: {
      width: 0,
      height: 1,
    },
    shadowOpacity: 0.22,
    shadowRadius: 2.22,
    elevation: 3,
  },
  modeButtonText: {
    fontSize: 14,
    fontWeight: '500',
    color: '#5f6368',
  },
  modeButtonTextActive: {
    color: '#1a73e8',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ddd',
    borderRadius: 8,
    padding: 16,
    fontSize: 16,
    marginBottom: 16,
    backgroundColor: '#fafafa',
  },
  button: {
    padding: 16,
    borderRadius: 8,
    alignItems: 'center',
    marginBottom: 12,
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  primaryButton: {
    backgroundColor: '#1a73e8',
  },
  googleButton: {
    backgroundColor: '#4285f4',
  },
  appleButton: {
    backgroundColor: '#000',
  },
  buttonText: {
    color: 'white',
    fontSize: 16,
    fontWeight: '600',
  },
  divider: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 20,
  },
  dividerLine: {
    flex: 1,
    height: 1,
    backgroundColor: '#ddd',
  },
  dividerText: {
    marginHorizontal: 16,
    color: '#888',
    fontSize: 14,
  },
  footer: {
    alignItems: 'center',
    marginTop: 40,
  },
  footerText: {
    color: '#888',
    fontSize: 14,
    textAlign: 'center',
  },
});

export default LoginScreen;
