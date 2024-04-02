import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable';
import { Connect } from '@/components/pages/Connect';
import { Button } from '@/components/ui/button';
import { Topics } from '@/components/pages/Topics';

function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string>('');
  const [screen, setScreen] = useState<string>('topics');

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
              onClick={() => setScreen('topics')}
            >
              Topics
            </Button>
            <Button
              variant={screen === 'schemaRegistry' ? 'outline' : 'default'}
              className="m-2 ml-0 w-full"
              onClick={() => setScreen('schemaRegistry')}
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
          <Topics />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
