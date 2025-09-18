import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

const API = import.meta.env.VITE_API || "http://localhost:8080";

export default function App() {
  const [status, setStatus] = useState<any>(null);
  const [blocks, setBlocks] = useState<any[]>([]);
  const [q, setQ] = useState("");
  const nav = useNavigate();

  useEffect(() => {
    fetch(`${API}/status`).then(r=>r.json()).then(setStatus);
    fetch(`${API}/blocks`).then(r=>r.json()).then(d=>setBlocks(d.blocks || []));
  }, []);

  const onSearch = async (e:any) => {
    e.preventDefault();
    const r = await fetch(`${API}/search/${q}`); 
    const j = await r.json();
    if (j.type === "block") nav(`/block/${j.data.height}`);
    else if (j.type === "tx") nav(`/tx/${j.data.txid}`);
    else nav(`/address/${encodeURIComponent(q)}`);
  };

  return (
    <>
      <form onSubmit={onSearch} style={{marginTop: 16, marginBottom: 16}}>
        <input value={q} onChange={e=>setQ(e.target.value)} placeholder="Search height / block hash / txid / address" style={{width: "70%"}}/>
        <button style={{marginLeft: 8}}>Search</button>
      </form>
      <p>Tip height: <b>{status?.tip_height ?? "—"}</b></p>
      <h2>Latest Blocks</h2>
      <table width="100%" cellPadding={6}>
        <thead><tr><th>Height</th><th>Hash</th><th>Txs</th><th>Time</th></tr></thead>
        <tbody>
          {blocks.map((b:any)=>(
            <tr key={b.height}>
              <td><Link to={`/block/${b.height}`}>#{b.height}</Link></td>
              <td>{(b.hash||"").slice(0,16)}…</td>
              <td>{b.tx_count}</td>
              <td>{new Date((b.timestamp||0)*1000).toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}
