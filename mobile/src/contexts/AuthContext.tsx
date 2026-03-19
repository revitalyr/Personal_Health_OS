import React, { createContext, useContext, useReducer, useEffect } from 'react';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { GoogleSignin } from '@react-native-google-signin/google-signin';
import auth from '@react-native-firebase/auth';
import { API_BASE_URL } from '../config/constants';

// Types
export interface UserProfile {
  id: string;
  name: string;
  relationship: 'self' | 'child' | 'parent' | 'spouse' | 'sibling' | 'other';
  dateOfBirth?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Account {
  id: string;
  email?: string;
  phone?: string;
  googleId?: string;
  appleId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface AuthState {
  isAuthenticated: boolean;
  account: Account | null;
  profiles: UserProfile[];
  currentProfile: UserProfile | null;
  loading: boolean;
  error: string | null;
}

type AuthAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'LOGIN_SUCCESS'; payload: { account: Account; profiles: UserProfile[] } }
  | { type: 'SET_CURRENT_PROFILE'; payload: UserProfile }
  | { type: 'ADD_PROFILE'; payload: UserProfile }
  | { type: 'LOGOUT' }
  | { type: 'CLEAR_ERROR' };

const initialState: AuthState = {
  isAuthenticated: false,
  account: null,
  profiles: [],
  currentProfile: null,
  loading: false,
  error: null,
};

const authReducer = (state: AuthState, action: AuthAction): AuthState => {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    case 'LOGIN_SUCCESS':
      return {
        ...state,
        isAuthenticated: true,
        account: action.payload.account,
        profiles: action.payload.profiles,
        currentProfile: action.payload.profiles.find(p => p.relationship === 'self') || action.payload.profiles[0] || null,
        loading: false,
        error: null,
      };
    case 'SET_CURRENT_PROFILE':
      return { ...state, currentProfile: action.payload };
    case 'ADD_PROFILE':
      return { 
        ...state, 
        profiles: [...state.profiles, action.payload],
        loading: false,
        error: null,
      };
    case 'LOGOUT':
      return {
        ...initialState,
        loading: false,
      };
    case 'CLEAR_ERROR':
      return { ...state, error: null };
    default:
      return state;
  }
};

interface AuthContextType extends AuthState {
  loginEmail: (email: string, password: string) => Promise<void>;
  loginGoogle: () => Promise<void>;
  loginApple: () => Promise<void>;
  sendOTP: (phone: string) => Promise<void>;
  verifyOTP: (phone: string, code: string) => Promise<void>;
  register: (email: string, password: string, name: string, dateOfBirth?: string) => Promise<void>;
  createProfile: (name: string, relationship: string, dateOfBirth?: string) => Promise<void>;
  setCurrentProfile: (profile: UserProfile) => void;
  logout: () => Promise<void>;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(authReducer, initialState);

  useEffect(() => {
    GoogleSignin.configure({
      webClientId: 'YOUR_GOOGLE_WEB_CLIENT_ID',
      offlineAccess: true,
    });
    loadStoredAuth();
  }, []);

  const loadStoredAuth = async () => {
    try {
      const tokens = await AsyncStorage.getItem('auth_tokens');
      if (tokens) {
        const { access_token } = JSON.parse(tokens);
        // Validate token and load user data
        await validateAndLoadUser(access_token);
      }
    } catch (error) {
      console.error('Error loading stored auth:', error);
    }
  };

  const validateAndLoadUser = async (token: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const response = await fetch(`${API_BASE_URL}/auth/profiles`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        const data = await response.json();
        // Load profiles to confirm authentication
        dispatch({ 
          type: 'LOGIN_SUCCESS', 
          payload: { 
            account: { id: 'temp' }, // Will be updated by API
            profiles: data.data 
          } 
        });
      } else {
        // Token invalid, remove it
        await AsyncStorage.removeItem('auth_tokens');
      }
    } catch (error) {
      console.error('Error validating token:', error);
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  };

  const storeTokens = async (tokens: any) => {
    try {
      await AsyncStorage.setItem('auth_tokens', JSON.stringify(tokens));
    } catch (error) {
      console.error('Error storing tokens:', error);
    }
  };

  const loginEmail = async (email: string, password: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const response = await fetch(`${API_BASE_URL}/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          provider: 'email',
          email,
          password,
        }),
      });

      const data = await response.json();
      
      if (response.ok) {
        await storeTokens(data.data);
        await loadProfiles(data.data.access_token);
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'Login failed' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    }
  };

  const loginGoogle = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      await GoogleSignin.hasPlayServices();
      const userInfo = await GoogleSignin.signIn();
      
      const response = await fetch(`${API_BASE_URL}/auth/login/google`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          id_token: userInfo.idToken,
        }),
      });

      const data = await response.json();
      
      if (response.ok) {
        await storeTokens(data.data);
        await loadProfiles(data.data.access_token);
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'Google login failed' });
      }
    } catch (error: any) {
      dispatch({ type: 'SET_ERROR', payload: error.message || 'Google login failed' });
    }
  };

  const loginApple = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      // Apple Sign In implementation would go here
      // For now, placeholder
      dispatch({ type: 'SET_ERROR', payload: 'Apple Sign In not implemented yet' });
    } catch (error: any) {
      dispatch({ type: 'SET_ERROR', payload: error.message || 'Apple login failed' });
    }
  };

  const sendOTP = async (phone: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const response = await fetch(`${API_BASE_URL}/auth/phone/send-otp`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ phone }),
      });

      const data = await response.json();
      
      if (response.ok) {
        dispatch({ type: 'SET_LOADING', payload: false });
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'Failed to send OTP' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    }
  };

  const verifyOTP = async (phone: string, code: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const response = await fetch(`${API_BASE_URL}/auth/phone/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ phone, code }),
      });

      const data = await response.json();
      
      if (response.ok) {
        await storeTokens(data.data);
        await loadProfiles(data.data.access_token);
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'OTP verification failed' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    }
  };

  const register = async (email: string, password: string, name: string, dateOfBirth?: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const response = await fetch(`${API_BASE_URL}/auth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password,
          name,
          date_of_birth: dateOfBirth,
        }),
      });

      const data = await response.json();
      
      if (response.ok) {
        await storeTokens(data.data);
        await loadProfiles(data.data.access_token);
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'Registration failed' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    }
  };

  const loadProfiles = async (token: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/auth/profiles`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      });

      if (response.ok) {
        const data = await response.json();
        dispatch({ 
          type: 'LOGIN_SUCCESS', 
          payload: { 
            account: { id: 'temp' }, // Will be updated by API
            profiles: data.data 
          } 
        });
      }
    } catch (error) {
      console.error('Error loading profiles:', error);
    }
  };

  const createProfile = async (name: string, relationship: string, dateOfBirth?: string) => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      
      const tokens = await AsyncStorage.getItem('auth_tokens');
      const { access_token } = JSON.parse(tokens || '{}');
      
      const response = await fetch(`${API_BASE_URL}/auth/profiles`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${access_token}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name,
          relationship,
          date_of_birth: dateOfBirth,
        }),
      });

      const data = await response.json();
      
      if (response.ok) {
        dispatch({ type: 'ADD_PROFILE', payload: data.data });
      } else {
        dispatch({ type: 'SET_ERROR', payload: data.message || 'Failed to create profile' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error' });
    }
  };

  const setCurrentProfile = (profile: UserProfile) => {
    dispatch({ type: 'SET_CURRENT_PROFILE', payload: profile });
  };

  const logout = async () => {
    try {
      await GoogleSignin.signOut();
      await auth().signOut();
      await AsyncStorage.removeItem('auth_tokens');
      dispatch({ type: 'LOGOUT' });
    } catch (error) {
      console.error('Error during logout:', error);
    }
  };

  const clearError = () => {
    dispatch({ type: 'CLEAR_ERROR' });
  };

  const value: AuthContextType = {
    ...state,
    loginEmail,
    loginGoogle,
    loginApple,
    sendOTP,
    verifyOTP,
    register,
    createProfile,
    setCurrentProfile,
    logout,
    clearError,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
