import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button.tsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select.tsx';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert.tsx';
import { AlertCircle, Check, ChevronsUpDown } from 'lucide-react';
import { toast } from 'sonner';
import { Input } from '@/components/ui/input.tsx';
import { useSettings } from '@/components/misc/SettingsProvider.tsx';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover.tsx';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command.tsx';
import { cn } from '@/lib/utils.ts';

export function Producer(props: { topic: string }) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [message, setMessage] = useState<string>(``);
  const [key, setKey] = useState<string>(``);
  const [state, setState] = useState<string>('avro');
  const [subjects, setSubjects] = useState<string[]>([]);
  const [selectedSubject, setSelectedSubject] = useState<string>('');
  const [schema, setSchema] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const { settings } = useSettings();

  async function produceMessage() {
    setError('');
    if (state === 'avro' && !selectedSubject) {
      setError('Please select a schema');
      return;
    }

    if (!message) {
      setError('Please enter a message');
      return;
    }

    const cleanedMessage = message.trim().replace(/[‘’“”]/g, function (match: any) {
      if (match === '‘' || match === '’') return "'";
      return '"';
    });
    const cleanedKey = key.trim().replace(/[‘’“”]/g, function (match: any) {
      if (match === '‘' || match === '’') return "'";
      return '"';
    });
    const action = state === 'avro' ? 'produce_message_avro' : 'produce_message_json';
    const payload =
      state === 'avro'
        ? { topic: props.topic, payload: cleanedMessage, schemaName: selectedSubject, key: cleanedKey }
        : { topic: props.topic, payload: cleanedMessage, key: cleanedKey };

    try {
      setIsLoading(true);
      await invoke(action, payload);
      toast.success('Message has been sent');
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
    if (!settings.schemaRegistryConnected) {
      setState('json');
    }
  }, [settings.schemaRegistryConnected]);

  useEffect(() => {
    setError('');
  }, [schema, state]);

  function handleInput(e: any) {
    setMessage(e.target.value);
  }

  function handleInputKey(e: any) {
    setKey(e.target.value);
  }

  return (
    <div className="p-6 w-full h-full">
      {settings.schemaRegistryConnected && (
        <Select onValueChange={value => setState(value)} defaultValue={state}>
          <SelectTrigger className="mb-6">
            <SelectValue />
          </SelectTrigger>

          <SelectContent className="mb-6">
            <SelectItem value="json">JSON</SelectItem>
            <SelectItem value="avro">AVRO</SelectItem>
          </SelectContent>
        </Select>
      )}

      {state === 'json' && (
        <>
          <Input className="w-full mb-6" onInput={handleInputKey} value={key} placeholder="Key" />
          <Textarea className="w-full" onInput={handleInput} value={message} placeholder="Enter message here" />
          <Button onClick={() => produceMessage()} disabled={isLoading} className="mt-6 mb-6">
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
              <Input className="w-full mb-6" onInput={handleInputKey} value={key} placeholder="Key" />
              <Textarea className="w-full" onInput={handleInput} value={message} placeholder="Enter message here" />
              <Button onClick={() => produceMessage()} disabled={isLoading} className="mt-6 mb-6">
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
              <Popover open={open} onOpenChange={setOpen}>
                <PopoverTrigger asChild>
                  <Button
                    variant="outline"
                    role="combobox"
                    aria-expanded={open}
                    className="w-full mb-6 justify-between"
                  >
                    {search ? subjects.find(subject => subject === search) : 'Select AVRO schema'}
                    <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-[400px] max-h-[200px] overflow-hidden p-0" side="bottom">
                  <Command>
                    <CommandInput placeholder="Select AVRO schema" />
                    <CommandList>
                      <CommandEmpty>No schema found</CommandEmpty>
                      <CommandGroup>
                        {subjects.map(subject => (
                          <CommandItem
                            key={subject + 'subjectSelectorKey'}
                            value={subject}
                            onSelect={currentValue => {
                              selectSchema(currentValue);
                              setSearch(currentValue);
                              setOpen(false);
                            }}
                          >
                            <Check className={cn('mr-2 h-4 w-4', search === subject ? 'opacity-100' : 'opacity-0')} />
                            {subject}
                          </CommandItem>
                        ))}
                      </CommandGroup>
                    </CommandList>
                  </Command>
                </PopoverContent>
              </Popover>

              {/*<Select onValueChange={v => selectSchema(v)}>*/}
              {/*  <SelectTrigger className="mb-6">*/}
              {/*    <SelectValue placeholder="Select AVRO schema" />*/}
              {/*  </SelectTrigger>*/}
              {/*  <SelectContent className="mb-6">*/}
              {/*    {subjects.map(subject => (*/}
              {/*      <SelectItem key={subject + 'subjectSelectorKey'} value={subject}>*/}
              {/*        {subject}*/}
              {/*      </SelectItem>*/}
              {/*    ))}*/}
              {/*  </SelectContent>*/}
              {/*</Select>*/}
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
