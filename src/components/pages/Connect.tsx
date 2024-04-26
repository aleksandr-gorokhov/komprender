import { Input } from '@/components/ui/input';
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card.tsx';
import { invoke } from '@tauri-apps/api/tauri';
import { Label } from '@/components/ui/label.tsx';

interface IConnection {
  kafka_broker: string;
  schema_registry?: string;
  name: string;
}

export function Connect({ onConnect, error }: { onConnect: Function; error?: string }) {
  const [broker, setBroker] = useState('');
  const [schemaRegistry, setSchemaRegistry] = useState('');
  const [name, setName] = useState('');

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
        {knownHosts.map(host => (
          <Card key={host.kafka_broker} className="w-[400px] ml-6 mt-10">
            <CardHeader>
              <CardTitle>{host.name}</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div className="flex items-center space-x-4 rounded-md">
                <p className="font-semibold">Kafka: </p>
                <p className="whitespace-nowrap truncate w-[150px]">{host.kafka_broker.split(',').join(' ')}</p>
              </div>
              {
                <div className="flex items-center space-x-4 rounded-md">
                  <p className="font-semibold">Schema Registry: </p>
                  <p className="whitespace-nowrap truncate w-[150px]">{host.schema_registry || 'Not connected'}</p>
                </div>
              }
            </CardContent>
            <CardFooter>
              <Button
                className="w-full"
                onClick={() =>
                  onConnect({ host: host.kafka_broker, name: host.name, schemaRegistry: host.schema_registry })
                }
              >
                Connect
              </Button>
            </CardFooter>
          </Card>
        ))}

        <div className="flex w-screen">
          <div className="w-1/2">
            <div className="grid w-full max-w-sm items-center gap-1.5 m-6">
              <Label htmlFor="name">Connection Name</Label>
              <Input id="name" placeholder="Connection Name" onChange={e => setName(e.target.value)} />
            </div>
            <div className="grid w-full max-w-sm items-center gap-1.5 m-6">
              <Label htmlFor="schemaRegistry">Schema Registry (optional)</Label>
              <Input
                id="schemaRegistry"
                placeholder="http://localhost:8081"
                onChange={e => setSchemaRegistry(e.target.value)}
              />
            </div>
            <div className="grid w-full max-w-sm items-center gap-1.5 m-6">
              <Label htmlFor="brokers">Kafka brokers</Label>
              <Input id="brokers" placeholder="kafka:9092" onChange={e => setBroker(e.target.value)} />
            </div>
            <Button className="m-6" onClick={() => onConnect({ host: broker, name, schemaRegistry })}>
              Connect
            </Button>
          </div>
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
