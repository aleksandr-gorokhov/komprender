import { createContext, ReactNode, useContext, useState } from 'react';

export interface ISettings {
  schemaRegistryConnected: boolean;
}

export interface ISettingsContextValue {
  settings: ISettings;
  setSettings: (settings: ISettings) => void;
}

const initialState = {
  settings: { schemaRegistryConnected: false },
  setSettings: () => null,
};

const SettingsProviderContext = createContext<ISettingsContextValue>(initialState);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<ISettings>({ schemaRegistryConnected: false });

  const value = {
    settings,
    setSettings,
  };
  return <SettingsProviderContext.Provider value={value}>{children}</SettingsProviderContext.Provider>;
}

export function useSettings() {
  const context = useContext(SettingsProviderContext);
  if (context === undefined) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}
