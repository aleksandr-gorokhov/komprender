import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable.tsx';
import { Input } from '@/components/ui/input.tsx';

interface ITopic {
  name: string;
  partitions: number;
  messages: number;
}

function App() {
  const [topics, setTopics] = useState<ITopic[]>([]);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    (async function fetchTopics() {
      const response = await invoke<ITopic[]>('fetch_topics', { filter });
      setTopics(response);
    })();
  }, [filter]);

  return (
    <div>
      <ResizablePanelGroup direction="horizontal" className="min-h-[200px] rounded-lg border">
        <ResizablePanel defaultSize={25}>
          <div className="flex h-full items-top justify-center p-6">
            <span className="font-semibold mt-6 p-6">Future Menu</span>
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
