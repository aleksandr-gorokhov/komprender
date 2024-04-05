import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button.tsx';
import { invoke } from '@tauri-apps/api/tauri';
import { useState } from 'react';

export function Producer(props: { topic: string }) {
  const [message, setMessage] = useState<string>(``);
  async function produceMessage() {
    await invoke('produce_message', { topic: props.topic, payload: message });
  }

  function handleInput(e: any) {
    setMessage(e.target.value);
  }

  return (
    <div className="p-6 w-full h-full">
      <Textarea className="w-full" onInput={handleInput} value={message} placeholder="Enter message here" />
      <Button onClick={() => produceMessage()} className="mt-6">
        Send
      </Button>
    </div>
  );
}
