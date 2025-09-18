import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
const API = import.meta.env.VITE_API || "http://localhost:8080";

export default function Address() {
  const { addr } = useParams();
  const [data, setData] = useState<any>(null);
  useEffect(()=>{ fetch(`${API}/address/${addr}`).then(r=>r.json()).then(setData); },[addr]);
  const txs = data?.txs || [];
  return (
    <div>
      <h2>Address</h2>
      <p><b>{addr}</b></p>
      <h3>Recent Transactions</h3>
      <table width="100%" cellPadding={6}>
        <thead><tr><th>Time</th><th>TxID</th><th>Block</th><th>From</th><th>To</th><th>Amount</th></tr></thead>
        <tbody>
          {txs.map((t:any)=>(
            <tr key={t.txid}>
              <td>{new Date((t.timestamp||0)*1000).toLocaleString()}</td>
              <td><Link to={`/tx/${t.txid}`}>{t.txid.slice(0,18)}…</Link></td>
              <td><Link to={`/block/${t.block_height}`}>#{t.block_height}</Link></td>
              <td>{t.from_addr || "—"}</td>
              <td>{t.to_addr || "—"}</td>
              <td>{t.amount}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
