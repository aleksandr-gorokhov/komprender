import { useParams } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { Separator } from '@/components/ui/separator.tsx';
import { Button } from '@/components/ui/button.tsx';
import { Producer } from '@/components/pages/Producer.tsx';
import { Consumer } from '@/components/pages/Consumer.tsx';

interface IPartition {
  id: number;
  high: number;
  low: number;
  leader: number;
  replicas: Array<number>;
  messages: number;
}

interface ITopicInfo {
  name: string;
  partitions: Array<IPartition>;
}

export function Topic() {
  let { name } = useParams();
  const [topic, setTopic] = useState<ITopicInfo | null>(null);
  const [state, setState] = useState<'partitions' | 'producer' | 'consumer'>('partitions');

  useEffect(() => {
    (async () => {
      const response = await invoke<ITopicInfo>('fetch_topic', { name });
      setTopic(response);
    })();
  }, [name]);

  async function openProducer() {
    setState('producer');
  }

  async function openConsumer() {
    setState('consumer');
  }

  if (!name) {
    return <div>404</div>;
  }

  if (!topic) {
    return <div className="flex h-full items-start justify-center pt-6">Loading...</div>;
  }

  return (
    <div className="flex flex-col h-full items-start justify-start pt-6">
      <div className="p-6 pb-4 border-b-2 w-full">
        <div className="space-y-1">
          <h4 className="text-sm font-medium leading-none">{topic.name}</h4>
        </div>
        <Separator className="my-4" />
        <div className="flex h-5 items-center space-x-4 text-sm">
          <div>{topic.partitions.length} partitions</div>
          <Separator orientation="vertical" />
          <div>{topic.partitions.reduce((acc, nextPartition) => acc + nextPartition.messages, 0)} messages</div>
          <Separator orientation="vertical" />
          <Button variant="secondary" onClick={() => openProducer()}>
            Produce message
          </Button>
          <Button variant="secondary" onClick={() => openConsumer()}>
            Consume messages
          </Button>
        </div>
      </div>

      {state === 'partitions' ? (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead colSpan={2} className="pl-6">
                <div className="flex items-center space-x-2">
                  <label
                    htmlFor="select-all"
                    className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                  >
                    ID
                  </label>
                </div>
              </TableHead>
              <TableHead className="text-center">Replicas</TableHead>
              <TableHead className="text-center">Offset start</TableHead>
              <TableHead className="text-center">Offset End</TableHead>
              <TableHead className="text-right pr-6">Messages</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {topic.partitions.map(partition => (
              <TableRow key={partition.id}>
                <TableCell colSpan={2} className="font-medium text-left  pl-6">
                  <div className="flex items-center space-x-2">
                    <p className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                      {partition.id}
                    </p>
                  </div>
                </TableCell>
                <TableCell className="text-center">
                  [
                  {partition.replicas.map(replica => (
                    <span key={replica + 'replica'} className={replica === partition.leader ? 'text-green-500' : ''}>
                      {replica}
                    </span>
                  ))}
                  ]
                </TableCell>
                <TableCell className="text-center pr-6">{partition.low}</TableCell>
                <TableCell className="text-center pr-6">{partition.high}</TableCell>
                <TableCell className="text-right pr-6">{partition.messages}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      ) : state === 'producer' ? (
        <Producer topic={name} />
      ) : (
        <Consumer topic={name} />
      )}
    </div>
  );
}
