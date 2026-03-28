import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  RefreshControl,
} from 'react-native';
import { healthCheck } from '../services/api';
import type { HealthResponse } from '../types';

export const DashboardScreen: React.FC = () => {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const fetchHealth = async () => {
    try {
      const data = await healthCheck();
      setHealth(data);
    } catch {
      setHealth(null);
    }
  };

  const onRefresh = async () => {
    setRefreshing(true);
    await fetchHealth();
    setRefreshing(false);
  };

  useEffect(() => {
    fetchHealth();
  }, []);

  return (
    <ScrollView
      style={styles.container}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
      }
    >
      <Text style={styles.title}>Dashboard</Text>
      <Text style={styles.subtitle}>Real-time ECU monitoring</Text>

      {/* Connection Status */}
      <View style={styles.card}>
        <View style={styles.statusRow}>
          <View
            style={[
              styles.statusDot,
              { backgroundColor: health ? '#22c55e' : '#ef4444' },
            ]}
          />
          <Text style={styles.statusText}>
            {health ? 'Connected' : 'Disconnected'}
          </Text>
        </View>
        {health && (
          <Text style={styles.versionText}>API v{health.version}</Text>
        )}
      </View>

      {/* Gauge Cards */}
      <View style={styles.gaugeGrid}>
        <GaugeCard title="Engine RPM" value={0} unit="rpm" color="#f59e0b" />
        <GaugeCard title="Speed" value={0} unit="km/h" color="#3b82f6" />
        <GaugeCard title="Coolant" value={0} unit="C" color="#ef4444" />
        <GaugeCard title="Intake" value={0} unit="kPa" color="#10b981" />
      </View>

      <View style={styles.card}>
        <Text style={styles.cardTitle}>Quick Stats</Text>
        <View style={styles.statsRow}>
          <StatItem label="ECU" value="0x7E0" />
          <StatItem label="PIDs" value="0" />
          <StatItem label="Frames" value="0" />
        </View>
      </View>
    </ScrollView>
  );
};

const GaugeCard: React.FC<{
  title: string;
  value: number;
  unit: string;
  color: string;
}> = ({ title, value, unit, color }) => (
  <View style={styles.gaugeCard}>
    <Text style={styles.gaugeTitle}>{title}</Text>
    <Text style={[styles.gaugeValue, { color }]}>{value.toFixed(1)}</Text>
    <Text style={styles.gaugeUnit}>{unit}</Text>
  </View>
);

const StatItem: React.FC<{ label: string; value: string }> = ({
  label,
  value,
}) => (
  <View style={styles.statItem}>
    <Text style={styles.statLabel}>{label}</Text>
    <Text style={styles.statValue}>{value}</Text>
  </View>
);

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#111827',
    padding: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    marginTop: 48,
  },
  subtitle: {
    fontSize: 14,
    color: '#9ca3af',
    marginBottom: 24,
  },
  card: {
    backgroundColor: '#1f2937',
    borderRadius: 12,
    padding: 16,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#374151',
  },
  cardTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
    marginBottom: 12,
  },
  statusRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  statusDot: {
    width: 10,
    height: 10,
    borderRadius: 5,
  },
  statusText: {
    fontSize: 14,
    color: '#d1d5db',
  },
  versionText: {
    fontSize: 12,
    color: '#6b7280',
    marginTop: 4,
  },
  gaugeGrid: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 12,
    marginBottom: 16,
  },
  gaugeCard: {
    backgroundColor: '#1f2937',
    borderRadius: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: '#374151',
    width: '48%',
    flexGrow: 1,
  },
  gaugeTitle: {
    fontSize: 12,
    color: '#9ca3af',
    marginBottom: 4,
  },
  gaugeValue: {
    fontSize: 28,
    fontWeight: 'bold',
  },
  gaugeUnit: {
    fontSize: 12,
    color: '#6b7280',
  },
  statsRow: {
    flexDirection: 'row',
    justifyContent: 'space-around',
  },
  statItem: {
    alignItems: 'center',
  },
  statLabel: {
    fontSize: 10,
    color: '#9ca3af',
    textTransform: 'uppercase',
  },
  statValue: {
    fontSize: 18,
    fontWeight: '600',
    color: '#fff',
    fontFamily: 'monospace',
    marginTop: 4,
  },
});
