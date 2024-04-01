import {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

function App() {
  const [topics, setTopics] = useState<string[]>([]);

  useEffect(() => {
    (async function fetchTopics() {
      const response = await invoke<string[]>("fetch_topics");
      setTopics(response);
    })();
  }, [])


  return (
    <div className="container">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead colSpan={3} className="w-[100px]">Topic</TableHead>
            <TableHead className="text-right">Amount</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {topics.map((topic) => (
            <TableRow key={topic}>
              <TableCell colSpan={3} className="font-medium text-left">{topic}</TableCell>
              <TableCell className="text-right">Ebalo</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

    </div>
  );
}

export default App;
