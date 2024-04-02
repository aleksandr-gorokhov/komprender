import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';
import { BrowserRouter } from 'react-router-dom';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <div className="absolute top-0 h-8 w-full" data-tauri-drag-region></div>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </React.StrictMode>
);
