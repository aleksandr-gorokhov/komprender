import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { Connect } from '@/components/pages/Connect';
import { Button } from '@/components/ui/button';
import { Topics } from '@/components/pages/Topics';
import { Route, Routes, useNavigate } from 'react-router-dom';
import { CreateTopic } from '@/components/pages/CreateTopic.tsx';
import { Topic } from '@/components/pages/Topic.tsx';
import { useSettings } from '@/components/misc/SettingsProvider.tsx';

function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string>('');
  const [screen, setScreen] = useState<string>('topics');
  const navigate = useNavigate();
  const { settings, setSettings } = useSettings();

  function connect(payload: { host: string; name: string; schemaRegistry?: string }) {
    (async () => {
      setError('');
      try {
        setConnecting(true);
        const result = await invoke<boolean>('connect', payload);
        navigate('/');
        setIsConnected(result);
        setSettings({ ...settings, schemaRegistryConnected: !!payload.schemaRegistry });
      } catch (err: any) {
        setError(err);
      } finally {
        setConnecting(false);
      }
    })();
  }

  function disconnect() {
    (async () => {
      setError('');
      try {
        await invoke<boolean>('disconnect');
        setIsConnected(false);
        navigate('/');
      } catch (err: any) {
        setError(err);
      }
    })();
  }

  if (!isConnected) {
    return <Connect onConnect={connect} error={error} connecting={connecting} />;
  }

  return (
    <div>
      <ResizablePanelGroup direction="horizontal" className="min-h-screen rounded-lg border relative">
        <ResizablePanel defaultSize={25}>
          <div className="flex flex-col h-screen w-full items-start justify-between p-10 top-0 left-0">
            <div className="w-full">
              <Button
                variant={screen === 'topics' ? 'outline' : 'default'}
                className="m-2 ml-0 w-full"
                onClick={() => {
                  setScreen('topics');
                  navigate('/topics');
                }}
              >
                Topics
              </Button>
              {settings.schemaRegistryConnected && false && (
                <Button
                  variant={screen === 'schema' ? 'outline' : 'default'}
                  className="m-2 ml-0 w-full"
                  onClick={() => {
                    setScreen('schema');
                    navigate('/schema');
                  }}
                >
                  Schema Registry
                </Button>
              )}
            </div>
            <Button variant="destructive" className="m-2 ml-0 w-full" onClick={disconnect}>
              Disconnect
            </Button>
          </div>
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={75}>
          <Routes>
            <Route path="/" Component={() => Topics({ disconnect })} />
            <Route path="/topics" Component={() => Topics({ disconnect })} />
            <Route path="/schema" Component={() => null} />
            <Route path="/topic/:name" Component={Topic} />
            <Route path="/create" Component={CreateTopic} />
          </Routes>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
