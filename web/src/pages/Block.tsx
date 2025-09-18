import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";

const API = import.meta.env.VITE_API || "http://localhost:8080";

export default function Block() {
  const { height } = useParams();
  const [data, setData] = useState<any>(null);

  useEffect(() => {
    fetch(`${API}/block/${height}`).then(r=>r.json()).then(setData);
  }, [height]);

  const b = data?.block;
  const txs = data?.transactions || [];

  return (
    <div>
      <h2>Block #{height}</h2>
      {!b ? <p>Loading…</p> :
        <>
          <p><b>Hash:</b> {b.hash}</p>
          <p><b>Parent:</b> {b.parent_hash}</p>
          <p><b>Time:</b> {new Date((b.timestamp||0)*1000).toLocaleString()}</p>
          <p><b>Txs:</b> {b.tx_count}</p>
        </>
      }
      <h3>Transactions</h3>
      <table width="100%" cellPadding={6}>
        <thead><tr><th>#</th><th>TxID</th><th>From</th><th>To</th><th>Amount</th><th>Fee</th></tr></thead>
        <tbody>
          {txs.map((t:any, i:number)=>(
            <tr key={t.txid}>
              <td>{i}</td>
              <td><Link to={`/tx/${t.txid}`}>{t.txid.slice(0,20)}…</Link></td>
              <td>{t.from_addr || "—"}</td>
              <td>{t.to_addr || "—"}</td>
              <td>{t.amount}</td>
              <td>{t.fee}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
