import React from 'react';
import { NavigationContainer, DarkTheme } from '@react-navigation/native';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { StatusBar } from 'expo-status-bar';
import { DashboardScreen } from './screens/DashboardScreen';
import { DtcScreen } from './screens/DtcScreen';
import { EcuScreen } from './screens/EcuScreen';

const Tab = createBottomTabNavigator();

const canaryTheme = {
  ...DarkTheme,
  colors: {
    ...DarkTheme.colors,
    primary: '#f59e0b',
    background: '#111827',
    card: '#1f2937',
    border: '#374151',
  },
};

export default function App() {
  return (
    <NavigationContainer theme={canaryTheme}>
      <StatusBar style="light" />
      <Tab.Navigator
        screenOptions={{
          headerShown: false,
          tabBarActiveTintColor: '#f59e0b',
          tabBarInactiveTintColor: '#6b7280',
          tabBarStyle: {
            backgroundColor: '#1f2937',
            borderTopColor: '#374151',
          },
        }}
      >
        <Tab.Screen
          name="Dashboard"
          component={DashboardScreen}
          options={{ tabBarLabel: 'Dashboard' }}
        />
        <Tab.Screen
          name="DTCs"
          component={DtcScreen}
          options={{ tabBarLabel: 'DTCs' }}
        />
        <Tab.Screen
          name="ECUs"
          component={EcuScreen}
          options={{ tabBarLabel: 'ECUs' }}
        />
      </Tab.Navigator>
    </NavigationContainer>
  );
}
