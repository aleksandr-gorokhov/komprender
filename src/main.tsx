import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';
import { BrowserRouter } from 'react-router-dom';
import { ThemeProvider } from '@/components/misc/ThemeProvider.tsx';
import { ThemeToggle } from '@/components/misc/ThemeToggle.tsx';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="absolute top-0 h-8 w-full" data-tauri-drag-region></div>
      <ThemeToggle></ThemeToggle>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </ThemeProvider>
  </React.StrictMode>
);
