import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import Icon from 'react-native-vector-icons/MaterialIcons';

import { AuthProvider, useAuth } from './contexts/AuthContext';
import LoginScreen from './screens/LoginScreen';
import ProfileSelectionScreen from './screens/ProfileSelectionScreen';
import DashboardScreen from './screens/DashboardScreen';
import TimelineScreen from './screens/TimelineScreen';
import DocumentsScreen from './screens/DocumentsScreen';
import ReportsScreen from './screens/ReportsScreen';
import ProfileScreen from './screens/ProfileScreen';

const Stack = createStackNavigator();
const Tab = createBottomTabNavigator();

const AppTabs: React.FC = () => {
  return (
    <Tab.Navigator
      screenOptions={({ route }) => ({
        tabBarIcon: ({ focused, color, size }) => {
          let iconName: string;

          if (route.name === 'Dashboard') {
            iconName = 'dashboard';
          } else if (route.name === 'Timeline') {
            iconName = 'timeline';
          } else if (route.name === 'Documents') {
            iconName = 'folder';
          } else if (route.name === 'Reports') {
            iconName = 'assessment';
          } else if (route.name === 'Profile') {
            iconName = 'person';
          } else {
            iconName = 'help';
          }

          return <Icon name={iconName} size={size} color={color} />;
        },
        tabBarActiveTintColor: '#1a73e8',
        tabBarInactiveTintColor: 'gray',
        headerShown: false,
      })}
    >
      <Tab.Screen 
        name="Dashboard" 
        component={DashboardScreen}
        options={{ title: 'Home' }}
      />
      <Tab.Screen 
        name="Timeline" 
        component={TimelineScreen}
        options={{ title: 'Timeline' }}
      />
      <Tab.Screen 
        name="Documents" 
        component={DocumentsScreen}
        options={{ title: 'Documents' }}
      />
      <Tab.Screen 
        name="Reports" 
        component={ReportsScreen}
        options={{ title: 'Reports' }}
      />
      <Tab.Screen 
        name="Profile" 
        component={ProfileScreen}
        options={{ title: 'Profile' }}
      />
    </Tab.Navigator>
  );
};

const AuthStack: React.FC = () => {
  const { isAuthenticated, currentProfile } = useAuth();

  return (
    <Stack.Navigator screenOptions={{ headerShown: false }}>
      {!isAuthenticated ? (
        <Stack.Screen name="Login" component={LoginScreen} />
      ) : !currentProfile ? (
        <Stack.Screen name="ProfileSelection" component={ProfileSelectionScreen} />
      ) : (
        <Stack.Screen name="Main" component={AppTabs} />
      )}
    </Stack.Navigator>
  );
};

const App: React.FC = () => {
  return (
    <AuthProvider>
      <NavigationContainer>
        <AuthStack />
      </NavigationContainer>
    </AuthProvider>
  );
};

export default App;
