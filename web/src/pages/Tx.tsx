import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
const API = import.meta.env.VITE_API || "http://localhost:8080";

export default function Tx() {
  const { txid } = useParams();
  const [data, setData] = useState<any>(null);
  useEffect(()=>{ fetch(`${API}/tx/${txid}`).then(r=>r.json()).then(setData); },[txid]);
  const t = data?.tx;
  return (
    <div>
      <h2>Transaction</h2>
      {!t ? <p>Loading…</p> :
        <>
          <p><b>TxID:</b> {t.txid}</p>
          <p><b>Block:</b> <Link to={`/block/${t.block_height}`}>#{t.block_height}</Link></p>
          <p><b>Time:</b> {new Date((t.timestamp||0)*1000).toLocaleString()}</p>
          <p><b>From:</b> {t.from_addr || "—"}</p>
          <p><b>To:</b> {t.to_addr || "—"}</p>
          <p><b>Amount:</b> {t.amount}</p>
          <p><b>Fee:</b> {t.fee}</p>
          <pre style={{background:"#f6f6f6", padding:12}}>{JSON.stringify(t, null, 2)}</pre>
        </>
      }
    </div>
  );
}
