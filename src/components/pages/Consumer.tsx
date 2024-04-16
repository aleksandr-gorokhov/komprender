import { Button } from '@/components/ui/button.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion.tsx';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';

interface IMessage {
  key: string;
  offset: number;
  partition: number;
  value: object;
}

export function Consumer(props: { topic: string }) {
  const [fromBeginning, setFromBeginning] = useState<boolean>(false);
  const [consumeLastX, setConsumeLastX] = useState<boolean>(false);
  const [newMessages, setNewMessages] = useState<boolean>(true);
  const [mode, setMode] = useState<'end' | 'beginning' | 'last'>('end');
  const [messages, setMessages] = useState<IMessage[]>([]);
  const [consuming, setConsuming] = useState<boolean>(false);

  useEffect(() => {
    return () => {
      stop();
    };
  }, []);
  useEffect(() => {
    let isActive = true;

    (async () => {
      const unsubscribe = await listen<IMessage>('message_received', event => {
        if (isActive) {
          setMessages(messages => [event.payload, ...messages].splice(0, 100));
        }
      });

      return () => {
        isActive = false;
        unsubscribe();
      };
    })();
  }, []);

  useEffect(() => {
    if (mode === 'end') {
      setFromBeginning(false);
      setConsumeLastX(false);
      setNewMessages(true);
      return;
    }

    if (mode === 'last') {
      setFromBeginning(false);
      setConsumeLastX(true);
      setNewMessages(false);
      return;
    }

    if (mode === 'beginning') {
      setFromBeginning(true);
      setConsumeLastX(false);
      setNewMessages(false);
      return;
    }
  }, [mode]);

  const label = fromBeginning
    ? 'Consume from beginning'
    : consumeLastX
      ? `Consume last 100 messages`
      : 'Consume new messages';

  async function consume() {
    setMessages([]);
    setConsuming(true);
    await invoke('consume_messages', {
      topic: props.topic,
      mode,
    });
    setConsuming(false);
  }

  async function stop() {
    await invoke('stop_consumers');
    setConsuming(false);
  }

  return (
    <div className="p-6 w-full h-full">
      <div className="mr-2 flex-col items-center p-2">
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="from-beginning" onCheckedChange={() => setMode('beginning')} checked={fromBeginning} />
          <label
            htmlFor="from-beginning"
            className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Consume first 100 messages (once)
          </label>
        </div>
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="new-messages" onCheckedChange={() => setMode('end')} checked={newMessages} />
          <label
            htmlFor="new-messages"
            className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Consume new messages (continuous)
          </label>
        </div>
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="consume-last-x" onCheckedChange={() => setMode('last')} checked={consumeLastX} />
          <label
            htmlFor="consume-last-x"
            className="flex items-center cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 flex-grow"
          >
            <span>Consume last 100 messages (continuous)</span>
          </label>
        </div>
        <div className="space-x-4">
          <Button disabled={consuming} onClick={() => consume()}>
            {label}
          </Button>
          <Button disabled={!consuming} onClick={() => stop()}>
            Stop consuming
          </Button>
        </div>
        <Accordion type="single" collapsible className="w-full mt-6 border border-b-0">
          {messages.map((message, i) => (
            <AccordionItem value={'item-' + i} key={'consumedMessageTable' + i} className="p-2">
              <AccordionTrigger>
                <p className="w-1/2 truncate">{JSON.stringify(message.value)}</p>
              </AccordionTrigger>
              <AccordionContent className="p-2">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead colSpan={1} className="text-left">
                        Partition
                      </TableHead>
                      <TableHead colSpan={1} className="text-left pr-6">
                        Offset
                      </TableHead>
                      <TableHead colSpan={1} className="text-left pr-6">
                        Key
                      </TableHead>
                      <TableHead colSpan={6} className="w-[100px] pl-0">
                        <div className="flex items-center space-x-2">Value</div>
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow key={'message' + i}>
                      <TableCell colSpan={1} className="text-left">
                        {message.partition}
                      </TableCell>
                      <TableCell colSpan={1} className="text-left pr-6">
                        {message.offset}
                      </TableCell>
                      <TableCell colSpan={1} className="text-left pr-6">
                        {message.key}
                      </TableCell>
                      <TableCell colSpan={6} className="font-medium text-left pl-0">
                        <div className="flex items-center space-x-2">
                          <p className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                            <pre>{JSON.stringify(message.value, null, 2)}</pre>
                          </p>
                        </div>
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </div>
    </div>
  );
}
