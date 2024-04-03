import { useParams } from 'react-router-dom';

export function Topic() {
  let { name } = useParams();
  return <div className="p-6">Topic {name}</div>;
}
