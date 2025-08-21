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
import { Delete, Add, Dehaze, FolderOpen } from '@mui/icons-material'; // Importa l'icona FolderOpen

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
                <Box
                    key={index}
                    sx={{
                        display: 'flex',
                        alignItems: 'center',
                        p: 1,
                        mb: 1,
                        gap: 1
                    }}
                >
                    {/* Frecce su/giù */}
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.5 }}>
                        <IconButton color="primary" onClick={() => alert("WIP")}>
                            <Dehaze />
                        </IconButton>
                    </Box>

                    {/* Pattern */}
                    <TextField
                        sx={{ flex: 4 }}
                        fullWidth
                        label="Pattern"
                        value={rule.pattern}
                        size='small'
                        onChange={(e) => handleChange(index, 'pattern', e.target.value)}
                    />

                    {/* Destinazione */}
                    <TextField
                        sx={{ flex: 6 }}
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
                                        sx={{ color: '#fff' }}
                                    >
                                        <FolderOpen />
                                    </IconButton>
                                </InputAdornment>
                            ),
                        }}
                    />

                    {/* Delete */}
                    <IconButton color="error" onClick={() => removeRule(index)}>
                        <Delete />
                    </IconButton>
                </Box>
            ))}

            <Button sx={{ ml: 1 }} variant="buttonPrimary" startIcon={<Add />} onClick={addRule}>
                Aggiungi Regola
            </Button>
            <Box sx={{ mt: 3, display: 'flex', justifyContent: 'right', gap: 2 }}>
                <Button variant="buttonPrimary" onClick={saveRules}>
                    Salva Regole
                </Button>
            </Box>
        </Paper>
    );
};

export default Rules;