import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/components/misc/ThemeProvider.tsx';
import { ThemeToggle } from '@/components/misc/ThemeToggle.tsx';
import { Toaster } from '@/components/ui/sonner.tsx';
import { SettingsProvider } from '@/components/misc/SettingsProvider.tsx';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <SettingsProvider>
        <div className="absolute top-0 h-8 w-full z-50" data-tauri-drag-region></div>
        <ThemeToggle></ThemeToggle>
        <BrowserRouter>
          <App />
          <Toaster closeButton />
        </BrowserRouter>
      </SettingsProvider>
    </ThemeProvider>
  </React.StrictMode>
);
