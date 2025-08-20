import React, { useState, useEffect } from 'react';

function App() {
  const [rules, setRules] = useState([]);

  useEffect(() => {
    window.electronAPI.readRules().then(setRules);
  }, []);

  const addRule = async () => {
    const newRule = { pattern: '*.txt', destination: 'C:\\Users\\Public\\Documents' };
    const updated = [...rules, newRule];
    setRules(updated);
    await window.electronAPI.writeRules(updated);
  };

  return (
    <div>
      <h1>OrdoProject - Gestione Regole</h1>
      <button onClick={addRule}>Aggiungi regola di esempio</button>
      <ul>
        {rules.map((r, i) => <li key={i}>{r.pattern} → {r.destination}</li>)}
      </ul>
    </div>
  );
}

export default App;
