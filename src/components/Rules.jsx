import React, { useState, useEffect } from 'react';
import {
    Typography,
    Button,
    TextField,
    IconButton,
    Box,
    Paper,
    Grid,
    InputAdornment, // Importa InputAdornment
} from '@mui/material';
import { Delete, Add, Save, ArrowUpward, ArrowDownward, FolderOpen } from '@mui/icons-material'; // Importa l'icona FolderOpen

const Rules = () => {
    const [rules, setRules] = useState([]);

    useEffect(() => {
        window.electronAPI.readRules().then((data) => {
            setRules(data || []);
        });
    }, []);

    const handleChange = (index, field, value) => {
        const updated = rules.map((rule, i) =>
            i === index ? { ...rule, [field]: value } : rule
        );
        setRules(updated);
    };

    const handleFolderPicker = async (index) => {
        const folderPath = await window.electronAPI.openFolderDialog();
        if (folderPath) {
          handleChange(index, 'destination', folderPath);
        }
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
        for (const rule of rules) {
            if (!rule.pattern || !rule.destination) {
                alert('Ogni regola deve avere pattern e destination.');
                return;
            }
        }
        // Simulazione di una chiamata ad electronAPI
        // Sostituisci con await window.electronAPI.writeRules(rules) nella tua app Electron
        console.log('Regole salvate:', rules);
        alert('Regole salvate!');
    };

    return (
        <Paper sx={{ p: 2, mb: 2 }} >
            {rules.map((rule, index) => (
                <Grid key={index} sx={{ p: 1, mb: 1 }} container spacing={2} alignItems="center">
                    <Grid item size={4}>
                        <TextField
                            fullWidth
                            label="Pattern"
                            value={rule.pattern}
                            size='small'
                            onChange={(e) => handleChange(index, 'pattern', e.target.value)}
                        />
                    </Grid>
                    <Grid item size={6}>
                        <TextField
                            fullWidth
                            label="Destinazione"
                            value={rule.destination}
                            size='small'
                            onChange={(e) => handleChange(index, 'destination', e.target.value)}
                            InputProps={{
                                endAdornment: (
                                    <InputAdornment position="end">
                                        <IconButton
                                            aria-label="seleziona cartella"
                                            onClick={() => handleFolderPicker(index)}
                                            edge="end"
                                        >
                                            <FolderOpen />
                                        </IconButton>
                                    </InputAdornment>
                                ),
                            }}
                        />
                    </Grid>
                    <Grid item size={2} sx={{ textAlign: { xs: 'right', sm: 'center' } }}>
                        <IconButton color="primary" onClick={() => alert("WIP")}>
                            <ArrowUpward />
                        </IconButton>
                        <IconButton color="primary" onClick={() => alert("WIP")}>
                            <ArrowDownward />
                        </IconButton>
                        <IconButton color="error" onClick={() => removeRule(index)}>
                            <Delete />
                        </IconButton>
                    </Grid>
                </Grid>
            ))}

            <Box sx={{ mt: 3, display: 'flex', justifyContent: 'center', gap: 2 }}>
                <Button variant="contained" startIcon={<Add />} onClick={addRule}>
                    Aggiungi Regola
                </Button>
                <Button variant="contained" color="success" startIcon={<Save />} onClick={saveRules}>
                    Salva Regole
                </Button>
            </Box>
        </Paper>
    );
};

export default Rules;