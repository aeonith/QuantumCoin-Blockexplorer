import React from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route, Link } from "react-router-dom";
import App from "./App";
import Block from "./pages/Block";
import Tx from "./pages/Tx";
import Address from "./pages/Address";

const Root = () => (
  <BrowserRouter>
    <div style={{maxWidth: 960, margin: "0 auto", padding: 24}}>
      <header style={{display: "flex", justifyContent: "space-between", alignItems: "center"}}>
        <h1 style={{margin: 0}}><Link to="/">QuantumCoin Explorer</Link></h1>
        <nav><a href="https://quantumcoincrypto.com" target="_blank">Website</a></nav>
      </header>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/block/:height" element={<Block />} />
        <Route path="/tx/:txid" element={<Tx />} />
        <Route path="/address/:addr" element={<Address />} />
      </Routes>
      <footer style={{marginTop: 48, opacity: 0.8}}>
        <small>API: {import.meta.env.VITE_API ?? "http://localhost:8080"} • © QuantumCoin</small>
      </footer>
    </div>
  </BrowserRouter>
);

createRoot(document.getElementById("root")!).render(<Root />);
