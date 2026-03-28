import React from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Sidebar } from './components/Sidebar';
import { Dashboard } from './pages/Dashboard';
import { DtcPage } from './pages/DtcPage';
import { EcuPage } from './pages/EcuPage';
import { LiveMonitor } from './pages/LiveMonitor';
import { Sessions } from './pages/Sessions';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 2,
    },
  },
});

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <div className="flex min-h-screen bg-gray-900 text-white">
          <Sidebar />
          <main className="flex-1 p-6 overflow-auto">
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/dtc" element={<DtcPage />} />
              <Route path="/ecus" element={<EcuPage />} />
              <Route path="/live" element={<LiveMonitor />} />
              <Route path="/sessions" element={<Sessions />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </QueryClientProvider>
  );
};

export default App;
