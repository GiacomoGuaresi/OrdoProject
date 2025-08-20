import React, { useState, useEffect } from 'react';
import {
  Container,
  Typography,
  Button,
  TextField,
  IconButton,
  Box,
  Paper,
  Grid
} from '@mui/material';
import { Delete, Add } from '@mui/icons-material';

function App() {
  const [rules, setRules] = useState([]);

  useEffect(() => {
    window.electronAPI.readRules().then(setRules);
  }, []);

  const handleChange = (index, field, value) => {
    const updated = rules.map((rule, i) =>
      i === index ? { ...rule, [field]: value } : rule
    );
    setRules(updated);
  };

  const addRule = () => {
    const newRule = { pattern: '', destination: '' };
    setRules([...rules, newRule]);
  };

  const removeRule = (index) => {
    const updated = rules.filter((_, i) => i !== index);
    setRules(updated);
  };

  const saveRules = async () => {
    // Validazione base: pattern e destination non vuoti
    for (const rule of rules) {
      if (!rule.pattern || !rule.destination) {
        alert("Ogni regola deve avere pattern e destination.");
        return;
      }
    }
    await window.electronAPI.writeRules(rules);
    alert("Regole salvate!");
  };

  return (
    <Container maxWidth="md" sx={{ mt: 4 }}>
      {rules.map((rule, index) => (
          <Grid key={index} sx={{ p: 1 }} container spacing={2} alignItems="center">
            <Grid item size={5}>
              <TextField
                fullWidth
                label="Pattern"
                value={rule.pattern}
                onChange={(e) => handleChange(index, 'pattern', e.target.value)}
              />
            </Grid>
            <Grid item size={6}>
              <TextField
                fullWidth
                label="Destination"
                value={rule.destination}
                onChange={(e) => handleChange(index, 'destination', e.target.value)}
              />
            </Grid>
            <Grid item size={1}>
              <IconButton color="error" onClick={() => removeRule(index)}>
                <Delete />
              </IconButton>
            </Grid>
          </Grid>
      ))}

      <Box sx={{ display: 'flex', gap: 2 }}>
        <Button variant="contained" startIcon={<Add />} onClick={addRule}>
          Aggiungi Regola
        </Button>
        <Button variant="contained" color="success" onClick={saveRules}>
          Salva Regole
        </Button>
      </Box>
    </Container>
  );
}

export default App;
