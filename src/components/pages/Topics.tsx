import { Input } from '@/components/ui/input.tsx';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import _ from 'lodash';

interface ITopic {
  name: string;
  partitions: number;
  messages: number;
}

export function Topics() {
  const [topics, setTopics] = useState<ITopic[]>([]);
  const [filter, setFilter] = useState('');
  const [checkedTopics, setCheckedTopics] = useState<string[]>([]);
  const [checkedAll, setCheckedAll] = useState(false);

  const fetchTopics = _.debounce(async () => {
    const response = await invoke<ITopic[]>('fetch_topics', { filter });
    setTopics(response);
  }, 100);

  useEffect(() => {
    fetchTopics();

    return () => {
      fetchTopics.cancel();
    };
  }, [filter]);

  function select(id: string) {
    const localCheckedTopics = [...checkedTopics];
    if (localCheckedTopics.includes(id)) {
      localCheckedTopics.splice(localCheckedTopics.indexOf(id), 1);
    } else {
      localCheckedTopics.push(id);
    }
    setCheckedTopics(localCheckedTopics);
  }

  function selectAll() {
    if (checkedAll) {
      setCheckedAll(false);
      setCheckedTopics([]);
      return;
    }
    setCheckedAll(true);
    setCheckedTopics(topics.map(topic => topic.name));
  }

  useEffect(() => {
    if (checkedTopics.length === topics.length && checkedTopics.length > 0) {
      setCheckedAll(true);
      return;
    }
    setCheckedAll(false);
  }, [checkedTopics]);

  return (
    <>
      <div className="flex items-center justify-center pt-6 pr-6 pl-6">
        <Input placeholder="Filter" onChange={e => setFilter(e.target.value)} />
      </div>

      <div className="flex h-full items-center justify-center p-6 pt-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead colSpan={2} className="w-[100px]">
                <div className="flex items-center space-x-2">
                  <Checkbox id="select-all" onCheckedChange={() => selectAll()} checked={checkedAll} className="mr-2" />
                  <label
                    htmlFor="select-all"
                    className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                  >
                    Topic
                  </label>
                </div>
              </TableHead>
              <TableHead className="text-right">Partitions</TableHead>
              <TableHead className="text-right">Messages</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {topics.map(topic => (
              <TableRow key={topic.name}>
                <TableCell colSpan={2} className="font-medium text-left">
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id={topic.name}
                      onCheckedChange={() => select(topic.name)}
                      checked={checkedTopics.includes('all') || checkedTopics.includes(topic.name)}
                      className="mr-2"
                    />
                    <label
                      htmlFor={topic.name}
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      {topic.name}
                    </label>
                  </div>
                </TableCell>
                <TableCell className="text-right">{topic.partitions}</TableCell>
                <TableCell className="text-right">{topic.messages}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </>
  );
}
