import { Input } from '@/components/ui/input';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from '@/components/ui/card.tsx';
export function Connect({ onConnect, error }: { onConnect: Function; error?: string }) {
  const [broker, setBroker] = useState('');

  const knownHosts = ['kafka:9092', 'kafka:9093'];

  return (
    <>
      <div className="flex flex-col items-start h-screen w-screen">
        {knownHosts.map(host => (
          <Card className="w-[380px] ml-6 mt-10">
            <CardHeader>
              <CardTitle>{host}</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <div className=" flex items-center space-x-4 rounded-md p-4"></div>
            </CardContent>
            <CardFooter>
              <Button className="w-full">Connect</Button>
            </CardFooter>
          </Card>
        ))}

        <div className="flex w-screen">
          <Input className="m-6" placeholder="Brokers" onChange={e => setBroker(e.target.value)} />
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
