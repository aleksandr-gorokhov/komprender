import { Button } from '@/components/ui/button.tsx';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import { useEffect, useState } from 'react';
import { Input } from '@/components/ui/input.tsx';

export function Consumer(props: { topic: string }) {
  const [fromBeginning, setFromBeginning] = useState<boolean>(false);
  const [consumeLastX, setConsumeLastX] = useState<boolean>(false);
  const [newMessages, setNewMessages] = useState<boolean>(true);
  const [mode, setMode] = useState<'new' | 'from' | 'last'>('new');
  const [lastMessagesCount, setLastMessagesCount] = useState<number>(100);

  useEffect(() => {
    if (mode === 'new') {
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

    if (mode === 'from') {
      setFromBeginning(true);
      setConsumeLastX(false);
      setNewMessages(false);
      return;
    }
  }, [mode]);

  const label = fromBeginning
    ? 'Consume from beginning'
    : consumeLastX
      ? `Consume last ${lastMessagesCount} messages`
      : 'Consume new messages';

  function preventString(e: any) {
    e.target.value = e.target.value.replace(/\D/g, '');
  }

  function consume() {
    console.log(props.topic, { fromBeginning, consumeLastX, newMessages, lastMessagesCount });
  }

  return (
    <div className="p-6 w-full h-full">
      <div className="mr-2 flex-col items-center p-2">
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="new-messages" onCheckedChange={() => setMode('new')} checked={newMessages} />
          <label
            htmlFor="new-messages"
            className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            Consume new messages
          </label>
        </div>
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="from-beginning" onCheckedChange={() => setMode('from')} checked={fromBeginning} />
          <label
            htmlFor="from-beginning"
            className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            From Beginning
          </label>
        </div>
        <div className="flex items-center space-x-2 w-full mb-6">
          <Checkbox id="consume-last-x" onCheckedChange={() => setMode('last')} checked={consumeLastX} />
          <label
            htmlFor="consume-last-x"
            className="flex items-center cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 flex-grow"
          >
            <span>Consume last</span>
            <Input
              className="mx-2 ml-2 mr-2 h-8 flex-shrink-0 w-20"
              value={lastMessagesCount}
              onInput={e => {
                preventString(e);
                const value = (e.target as any).value;
                setLastMessagesCount(Number(value));
              }}
            />{' '}
            <span>messages</span>
          </label>
        </div>
        <Button onClick={() => consume()}>{label}</Button>
      </div>
    </div>
  );
}
