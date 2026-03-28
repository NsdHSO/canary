import React, { useCallback, useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  StyleSheet,
  TouchableOpacity,
  RefreshControl,
  Alert,
} from 'react-native';
import { readDtcs, clearDtcs } from '../services/api';
import type { DtcResponse } from '../types';

const severityColors: Record<string, string> = {
  high: '#ef4444',
  medium: '#f59e0b',
  low: '#22c55e',
};

export const DtcScreen: React.FC = () => {
  const [dtcs, setDtcs] = useState<DtcResponse[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchDtcs = useCallback(async () => {
    setLoading(true);
    try {
      const data = await readDtcs();
      setDtcs(data);
    } catch {
      Alert.alert('Error', 'Failed to fetch DTCs');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDtcs();
  }, [fetchDtcs]);

  const handleClear = (code: string) => {
    Alert.alert('Clear DTC', `Clear ${code}?`, [
      { text: 'Cancel', style: 'cancel' },
      {
        text: 'Clear',
        style: 'destructive',
        onPress: async () => {
          try {
            await clearDtcs({ ecu_id: 0x7e0, codes: [code] });
            setDtcs((prev) => prev.filter((d) => d.code !== code));
          } catch {
            Alert.alert('Error', 'Failed to clear DTC');
          }
        },
      },
    ]);
  };

  const renderDtc = ({ item }: { item: DtcResponse }) => (
    <TouchableOpacity
      style={styles.dtcCard}
      onLongPress={() => handleClear(item.code)}
    >
      <View style={styles.dtcHeader}>
        <Text style={styles.dtcCode}>{item.code}</Text>
        <View
          style={[
            styles.severityBadge,
            { backgroundColor: (severityColors[item.severity] || '#6b7280') + '30' },
          ]}
        >
          <Text
            style={[
              styles.severityText,
              { color: severityColors[item.severity] || '#6b7280' },
            ]}
          >
            {item.severity.toUpperCase()}
          </Text>
        </View>
      </View>
      <Text style={styles.dtcDescription}>{item.description}</Text>
      <View style={styles.dtcFooter}>
        <Text style={styles.dtcSystem}>{item.system}</Text>
        <Text style={styles.dtcStatus}>
          Status: 0x{item.status.toString(16).toUpperCase().padStart(2, '0')}
        </Text>
      </View>
    </TouchableOpacity>
  );

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Diagnostic Trouble Codes</Text>
      <Text style={styles.subtitle}>
        {dtcs.length} DTC{dtcs.length !== 1 ? 's' : ''} found - Long press to clear
      </Text>
      <FlatList
        data={dtcs}
        renderItem={renderDtc}
        keyExtractor={(item) => item.code}
        refreshControl={
          <RefreshControl refreshing={loading} onRefresh={fetchDtcs} />
        }
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          !loading ? (
            <Text style={styles.emptyText}>No DTCs found. Vehicle is healthy.</Text>
          ) : null
        }
      />
    </View>
  );
};

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
    marginBottom: 16,
  },
  list: {
    paddingBottom: 32,
  },
  dtcCard: {
    backgroundColor: '#1f2937',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#374151',
  },
  dtcHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  dtcCode: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#f59e0b',
    fontFamily: 'monospace',
  },
  severityBadge: {
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  severityText: {
    fontSize: 10,
    fontWeight: '600',
  },
  dtcDescription: {
    fontSize: 14,
    color: '#d1d5db',
    marginBottom: 8,
  },
  dtcFooter: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  dtcSystem: {
    fontSize: 12,
    color: '#6b7280',
  },
  dtcStatus: {
    fontSize: 12,
    color: '#6b7280',
    fontFamily: 'monospace',
  },
  emptyText: {
    textAlign: 'center',
    color: '#6b7280',
    fontSize: 16,
    marginTop: 48,
  },
});
