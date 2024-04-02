import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { Connect } from '@/components/pages/Connect';
import { Button } from '@/components/ui/button';
import { Topics } from '@/components/pages/Topics';
import { Route, Routes, useNavigate } from 'react-router-dom';
import { TestContent } from '@/components/pages/TestContent.tsx';
import { CreateTopic } from '@/components/pages/CreateTopic.tsx';

function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string>('');
  const [screen, setScreen] = useState<string>('topics');
  const navigate = useNavigate();

  function connect(broker: string) {
    (async () => {
      setError('');
      try {
        const result = await invoke<boolean>('connect', { broker });
        setIsConnected(result);
      } catch (err: any) {
        setError(err);
      }
    })();
  }

  function disconnect() {
    (async () => {
      setError('');
      try {
        await invoke<boolean>('disconnect');
        setIsConnected(false);
      } catch (err: any) {
        setError(err);
      }
    })();
  }

  if (!isConnected) {
    return <Connect onConnect={connect} error={error} />;
  }

  return (
    <div>
      <ResizablePanelGroup direction="horizontal" className="min-h-[200px] rounded-lg border">
        <ResizablePanel defaultSize={25}>
          <div className="flex flex-col h-full items-start justify-start p-10">
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

            <Button variant="destructive" className="m-2 ml-0 w-full" onClick={disconnect}>
              Disconnect
            </Button>
          </div>
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={75}>
          <Routes>
            <Route path="/" Component={Topics} />
            <Route path="/topics" Component={Topics} />
            <Route path="/schema" Component={TestContent} />
            <Route path="/create" Component={CreateTopic} />
          </Routes>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
