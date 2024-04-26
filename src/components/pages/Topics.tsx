import { Input } from '@/components/ui/input.tsx';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table.tsx';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Checkbox } from '@/components/ui/checkbox.tsx';
import _ from 'lodash';
import { Button } from '@/components/ui/button.tsx';
import { useNavigate } from 'react-router-dom';
import { Plus } from 'lucide-react';
import { Skeleton } from '@/components/ui/skeleton.tsx';

interface ITopic {
  name: string;
  partitions: number;
  messages: number;
}

export function Topics({ disconnect }: { disconnect: () => void }) {
  const [topics, setTopics] = useState<ITopic[] | null>(null);
  const [filter, setFilter] = useState('');
  const [checkedTopics, setCheckedTopics] = useState<string[]>([]);
  const [checkedAll, setCheckedAll] = useState(false);
  const navigate = useNavigate();

  const fetchTopics = _.debounce(async (i = 0) => {
    try {
      const response = await invoke<ITopic[]>('fetch_topics', { filter });
      setTopics(response);
    } catch (err) {
      if (i > 5) {
        disconnect();
        return;
      }
      await new Promise(resolve => setTimeout(resolve, 1000));
      fetchTopics(i + 1);
    }
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
    setCheckedTopics(topics?.map(topic => topic.name) || []);
  }

  useEffect(() => {
    if (checkedTopics.length === topics?.length && checkedTopics.length > 0) {
      setCheckedAll(true);
      return;
    }
    setCheckedAll(false);
  }, [checkedTopics]);

  function dropTopics() {
    (async () => {
      await invoke('drop_topics', { topicNames: checkedTopics });
      fetchTopics();
    })();
  }

  return (
    <>
      <div className="flex items-center justify-center p-6 pb-0">
        <Input placeholder="Filter" onChange={e => setFilter(e.target.value)} />
      </div>

      <div className="flex items-start justify-start p-6 pb-0">
        <Button disabled={!checkedTopics.length} onClick={dropTopics}>
          Delete selected topics
        </Button>
        <Button className="ml-2 p-2" onClick={() => navigate('/create')}>
          <Plus size={20} />
        </Button>
      </div>

      <div className="flex h-full items-start justify-center pt-6">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead colSpan={3} className="w-[100px] pl-0">
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id="select-all"
                    onCheckedChange={() => selectAll()}
                    checked={checkedAll}
                    disabled={!topics?.length}
                    className="mr-2 ml-6"
                  />
                  <label
                    htmlFor="select-all"
                    className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                  >
                    Topic
                  </label>
                </div>
              </TableHead>
              <TableHead colSpan={1} className="text-right">
                Partitions
              </TableHead>
              <TableHead colSpan={1} className="text-right pr-6">
                Total Offset
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {topics
              ? topics?.map(topic => (
                  <TableRow key={topic.name} onClick={() => navigate(`/topic/${topic.name}`)}>
                    <TableCell colSpan={3} className="font-medium text-left pl-0">
                      <div className="flex items-center space-x-2">
                        <Checkbox
                          id={topic.name}
                          onClick={e => e.stopPropagation()}
                          onCheckedChange={() => select(topic.name)}
                          checked={checkedTopics.includes('all') || checkedTopics.includes(topic.name)}
                          className="mr-2 ml-6"
                        />
                        <p className="cursor-pointer text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                          {topic.name}
                        </p>
                      </div>
                    </TableCell>
                    <TableCell colSpan={1} className="text-right">
                      {topic.partitions}
                    </TableCell>
                    <TableCell colSpan={1} className="text-right pr-6">
                      {topic.messages}
                    </TableCell>
                  </TableRow>
                ))
              : Array.from({ length: 20 }).map((_, i) => (
                  <TableRow key={'skeletonNum' + i}>
                    <TableCell colSpan={3} className="font-medium text-left pl-0">
                      <div className="flex items-center space-x-2 pt-1 pb-1">
                        <Skeleton className="mr-2 ml-6 h-4 w-5 rounded-md" />
                        <Skeleton className="h-4 w-[300px]" />
                      </div>
                    </TableCell>
                    <TableCell colSpan={1} className="text-right">
                      <Skeleton className="h-4" />
                    </TableCell>
                    <TableCell colSpan={1} className="text-right pr-6">
                      <Skeleton className="h-4" />
                    </TableCell>
                  </TableRow>
                ))}
          </TableBody>
        </Table>
      </div>
    </>
  );
}
