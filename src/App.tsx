import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable.tsx';
import { Input } from '@/components/ui/input.tsx';
import { Connect } from '@/components/pages/Connect.tsx';
import { Button } from '@/components/ui/button.tsx';

interface ITopic {
  name: string;
  partitions: number;
  messages: number;
}

function App() {
  const [topics, setTopics] = useState<ITopic[]>([]);
  const [filter, setFilter] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    if (!isConnected) {
      return;
    }
    (async function fetchTopics() {
      const response = await invoke<ITopic[]>('fetch_topics', { filter });
      setTopics(response);
    })();
  }, [filter, isConnected]);

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
      console.error('trying to disconnect');
      setError('');
      try {
        await invoke<boolean>('disconnect');
        console.error('disconnected');
        setIsConnected(false);
      } catch (err: any) {
        console.error(err);
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
          <div className="flex h-full items-top justify-center p-6">
            <Button variant="destructive" onClick={disconnect}>
              Disconnect
            </Button>
          </div>
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={75}>
          <>
            <div className="flex items-center justify-center pt-6 pr-6 pl-6">
              <Input placeholder="Filter" onChange={e => setFilter(e.target.value)} />
            </div>

            <div className="flex h-full items-center justify-center p-6 pt-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead colSpan={2} className="w-[100px]">
                      Topic
                    </TableHead>
                    <TableHead className="text-right">Partitions</TableHead>
                    <TableHead className="text-right">Messages</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {topics.map(topic => (
                    <TableRow key={topic.name}>
                      <TableCell colSpan={2} className="font-medium text-left">
                        {topic.name}
                      </TableCell>
                      <TableCell className="text-right">{topic.partitions}</TableCell>
                      <TableCell className="text-right">{topic.messages}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
