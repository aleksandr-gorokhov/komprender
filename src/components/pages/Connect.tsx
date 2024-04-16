import { Input } from '@/components/ui/input';
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { invoke } from '@tauri-apps/api/tauri';

interface IConnection {
  kafka_broker: string;
  schema_registry?: string;
  name: string;
}

export function Connect({ onConnect, error }: { onConnect: Function; error?: string }) {
  const [broker, setBroker] = useState('');

  const [knownHosts, setKnownHosts] = useState<IConnection[] | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const hosts = await invoke<IConnection[]>('fetch_saved_brokers');
        setKnownHosts(hosts);
      } catch (e) {
        setKnownHosts([]);
        console.error(e);
      }
    })();
  }, []);

  if (!knownHosts) {
    return <div className="flex flex-row flex-wrap items-start w-screen pt-6">Loading...</div>;
  }

  return (
    <>
      <div className="flex flex-row flex-wrap items-start w-screen pt-6">
        {/*<div className="flex">*/}
        {knownHosts.map(host => (
          <Card key={host.kafka_broker} className="w-[500px] ml-6 mt-10">
            <CardHeader>
              <CardTitle>{host.name}</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div className="flex items-center space-x-4 rounded-md">
                <p className="font-semibold">Kafka: </p>
                <p>{host.kafka_broker.split(',').join(' ')}</p>
              </div>
              {host.schema_registry && (
                <div className="flex items-center space-x-4 rounded-md">
                  <p className="font-semibold">Schema Registry: </p>
                  <p>{host.schema_registry}</p>
                </div>
              )}
            </CardContent>
            <CardFooter>
              <Button className="w-full" onClick={() => onConnect(host.kafka_broker)}>
                Connect
              </Button>
            </CardFooter>
          </Card>
        ))}
        {/*</div>*/}

        <div className="flex w-screen">
          <Input className="m-6" placeholder="New Brokers" onChange={e => setBroker(e.target.value)} />
          <Button className="m-6 ml-0" onClick={() => onConnect(broker)}>
            Connect
          </Button>
        </div>
        {error && (
          <div className="mb-4 ml-6 min-w-72">
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          </div>
        )}
      </div>
    </>
  );
}
