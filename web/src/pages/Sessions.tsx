import React, { useState } from 'react';
import { createSession } from '../services/api';
import type { SessionResponse } from '../types';

export const Sessions: React.FC = () => {
  const [sessions, setSessions] = useState<SessionResponse[]>([]);
  const [creating, setCreating] = useState(false);

  const handleCreateSession = async () => {
    setCreating(true);
    try {
      const session = await createSession({
        ecu_id: 0x7e0,
        session_type: 'extended',
      });
      setSessions((prev) => [session, ...prev]);
    } catch (e) {
      console.error('Failed to create session:', e);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Diagnostic Sessions</h2>
          <p className="text-gray-400 text-sm">{sessions.length} session(s)</p>
        </div>
        <button
          onClick={handleCreateSession}
          disabled={creating}
          className="px-4 py-2 bg-canary-600 hover:bg-canary-700 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
        >
          {creating ? 'Creating...' : 'New Session'}
        </button>
      </div>

      {sessions.length === 0 ? (
        <div className="text-center py-16 text-gray-400">
          <p className="text-lg">No sessions yet</p>
          <p className="text-sm mt-2">Create a new diagnostic session to get started</p>
        </div>
      ) : (
        <div className="space-y-3">
          {sessions.map((session) => (
            <div
              key={session.session_id}
              className="bg-gray-800 rounded-lg p-4 border border-gray-700 flex items-center justify-between"
            >
              <div>
                <p className="font-mono text-sm text-canary-400">
                  {session.session_id}
                </p>
                <p className="text-sm text-gray-400 mt-1">
                  ECU 0x{session.ecu_id.toString(16).toUpperCase()} - {session.session_type}
                </p>
              </div>
              <div className="flex items-center gap-3">
                <span
                  className={`px-2 py-1 rounded text-xs ${
                    session.status === 'active'
                      ? 'bg-green-900/30 text-green-400'
                      : 'bg-gray-700 text-gray-400'
                  }`}
                >
                  {session.status}
                </span>
                <span className="text-xs text-gray-500">
                  {new Date(session.created_at).toLocaleString()}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
