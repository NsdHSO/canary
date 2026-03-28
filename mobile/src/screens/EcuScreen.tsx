import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  FlatList,
  StyleSheet,
  TextInput,
  RefreshControl,
} from 'react-native';
import { listEcus } from '../services/api';
import type { EcuInfo } from '../types';

export const EcuScreen: React.FC = () => {
  const [ecus, setEcus] = useState<EcuInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('');

  const fetchEcus = async () => {
    setLoading(true);
    try {
      const data = await listEcus(filter || undefined);
      setEcus(data.items);
    } catch {
      // handle error
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEcus();
  }, [filter]);

  const renderEcu = ({ item }: { item: EcuInfo }) => (
    <View style={styles.ecuCard}>
      <View style={styles.ecuHeader}>
        <View>
          <Text style={styles.ecuName}>
            {item.manufacturer} {item.model}
          </Text>
          <Text style={styles.ecuYear}>{item.year_range}</Text>
        </View>
        <View style={styles.typeBadge}>
          <Text style={styles.typeText}>{item.ecu_type}</Text>
        </View>
      </View>
      <Text style={styles.canId}>
        CAN ID: 0x{item.can_id.toString(16).toUpperCase()}
      </Text>
      <View style={styles.protocols}>
        {item.protocols.map((p) => (
          <View key={p} style={styles.protocolBadge}>
            <Text style={styles.protocolText}>{p}</Text>
          </View>
        ))}
      </View>
    </View>
  );

  return (
    <View style={styles.container}>
      <Text style={styles.title}>ECU Database</Text>
      <TextInput
        style={styles.searchInput}
        placeholder="Filter by manufacturer..."
        placeholderTextColor="#6b7280"
        value={filter}
        onChangeText={setFilter}
      />
      <FlatList
        data={ecus}
        renderItem={renderEcu}
        keyExtractor={(item) => item.id}
        refreshControl={
          <RefreshControl refreshing={loading} onRefresh={fetchEcus} />
        }
        contentContainerStyle={styles.list}
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
    marginBottom: 16,
  },
  searchInput: {
    backgroundColor: '#1f2937',
    borderRadius: 8,
    paddingHorizontal: 16,
    paddingVertical: 12,
    color: '#fff',
    fontSize: 14,
    borderWidth: 1,
    borderColor: '#374151',
    marginBottom: 16,
  },
  list: {
    paddingBottom: 32,
  },
  ecuCard: {
    backgroundColor: '#1f2937',
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#374151',
  },
  ecuHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 8,
  },
  ecuName: {
    fontSize: 16,
    fontWeight: '600',
    color: '#fff',
  },
  ecuYear: {
    fontSize: 12,
    color: '#9ca3af',
    marginTop: 2,
  },
  typeBadge: {
    backgroundColor: '#78350f30',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  typeText: {
    fontSize: 10,
    fontWeight: '600',
    color: '#f59e0b',
    fontFamily: 'monospace',
  },
  canId: {
    fontSize: 13,
    color: '#9ca3af',
    fontFamily: 'monospace',
    marginBottom: 8,
  },
  protocols: {
    flexDirection: 'row',
    gap: 6,
  },
  protocolBadge: {
    backgroundColor: '#374151',
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 4,
  },
  protocolText: {
    fontSize: 11,
    color: '#d1d5db',
  },
});
