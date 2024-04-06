import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button.tsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select.tsx';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert.tsx';
import { AlertCircle } from 'lucide-react';

export function Producer(props: { topic: string }) {
  const [message, setMessage] = useState<string>(``);
  const [state, setState] = useState<string>('avro');
  const [subjects, setSubjects] = useState<string[]>([]);
  const [selectedSubject, setSelectedSubject] = useState<string>('');
  const [schema, setSchema] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  async function produceMessageAvro() {
    setError('');
    if (!selectedSubject) {
      setError('Please select a schema');
      return;
    }
    if (!message) {
      setError('Please enter a message');
      return;
    }
    try {
      setIsLoading(true);
      await invoke('produce_message_avro', { topic: props.topic, payload: message, schemaName: selectedSubject });
    } catch (err) {
      if (typeof err === 'string') {
        setError(err);
        return;
      }

      if (err instanceof Error) {
        setError(err.message);
        return;
      }

      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function produceMessageJson() {
    if (!message) {
      setError('Please enter a message');
      return;
    }
    try {
      setIsLoading(true);
      await invoke('produce_message_json', { topic: props.topic, payload: message });
    } catch (err) {
      if (typeof err === 'string') {
        setError(err);
        return;
      }

      if (err instanceof Error) {
        setError(err.message);
        return;
      }

      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }

  async function selectSchema(subject: string) {
    try {
      const schemaData = await invoke<string>('fetch_schema', { subject });
      setSelectedSubject(subject);
      setSchema(JSON.stringify(JSON.parse(schemaData), undefined, 2));
    } catch (err) {
      console.error(err);
    }
  }

  useEffect(() => {
    if (state !== 'avro' || subjects.length) {
      return;
    }
    (async () => {
      const subjects = await invoke<string[]>('fetch_sr_subjects');
      setSubjects(subjects);
    })();
  }, [state]);

  useEffect(() => {
    setError('');
  }, [schema, state]);

  function handleInput(e: any) {
    setMessage(
      e.target.value.replace(/[‘’“”]/g, function (match: any) {
        if (match === '‘' || match === '’') return "'";
        return '"';
      })
    );
  }

  return (
    <div className="p-6 w-full h-full">
      <Select onValueChange={value => setState(value)} defaultValue={state}>
        <SelectTrigger className="mb-6">
          <SelectValue />
        </SelectTrigger>
        <SelectContent className="mb-6">
          <SelectItem value="json">JSON</SelectItem>
          <SelectItem value="avro">AVRO</SelectItem>
        </SelectContent>
      </Select>
      {state === 'json' && (
        <>
          <Textarea className="w-full" onInput={handleInput} value={message} placeholder="Enter message here" />
          <Button onClick={() => produceMessageJson()} disabled={isLoading} className="mt-6 mb-6">
            Send
          </Button>
          {error && (
            <div className="min-w-72">
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Error</AlertTitle>
                <AlertDescription>{error}</AlertDescription>
              </Alert>
            </div>
          )}
        </>
      )}
      {state === 'avro' && (
        <>
          <div className="w-full mr-6 mb-6 p-4 bg-yellow-100 text-yellow-900 rounded-md">
            Avro producer type is experimental. Library that encodes values to avro is of version 0.4.0. <br />
            <b>Messages may pass validation checks but still experience partial corruption during delivery.</b>
            <br />
            Please use with caution.
          </div>
          <div className="flex flex-row w-full">
            <div className="flex flex-col w-1/2 mr-6">
              <Textarea className="w-full" onInput={handleInput} value={message} placeholder="Enter message here" />
              <Button onClick={() => produceMessageAvro()} disabled={isLoading} className="mt-6 mb-6">
                Send
              </Button>
              {error && (
                <div className="min-w-72">
                  <Alert variant="destructive">
                    <AlertCircle className="h-4 w-4" />
                    <AlertTitle>Error</AlertTitle>
                    <AlertDescription>{error}</AlertDescription>
                  </Alert>
                </div>
              )}
            </div>
            <div className="flex flex-col w-1/2">
              <Select onValueChange={v => selectSchema(v)}>
                <SelectTrigger className="mb-6">
                  <SelectValue placeholder="Select AVRO schema" />
                </SelectTrigger>
                <SelectContent className="mb-6">
                  {subjects.map(subject => (
                    <SelectItem value={subject}>{subject}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {schema && (
                <div className="border rounded-md">
                  <div className="font-mono text-sm overflow-auto rounded-lg">
                    <pre className="p-4">{schema}</pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
