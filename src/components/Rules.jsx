import React, { useState, useEffect } from 'react';
import {
    Typography,
    Button,
    TextField,
    IconButton,
    Box,
    Paper,
    InputAdornment,
} from '@mui/material';
import { Delete, Add, Dehaze, FolderOpen } from '@mui/icons-material';
import { DragDropContext, Droppable, Draggable } from '@hello-pangea/dnd';

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
        console.log('Regole salvate:', rules);
        alert('Regole salvate!');
    };

    // Funzione chiamata al rilascio di un drag
    const onDragEnd = (result) => {
        if (!result.destination) return;
        const updated = Array.from(rules);
        const [moved] = updated.splice(result.source.index, 1);
        updated.splice(result.destination.index, 0, moved);
        setRules(updated);
    };

    return (
        <Paper sx={{ p: 2, mb: 2 }}>
            <DragDropContext onDragEnd={onDragEnd}>
                <Droppable droppableId="rulesList">
                    {(provided) => (
                        <div {...provided.droppableProps} ref={provided.innerRef}>
                            {rules.map((rule, index) => (
                                <Draggable key={index} draggableId={`rule-${index}`} index={index}>
                                    {(provided, snapshot) => (
                                        <Box
                                            ref={provided.innerRef}
                                            {...provided.draggableProps}
                                            sx={{
                                                display: 'flex',
                                                alignItems: 'center',
                                                p: 1,
                                                mb: 1,
                                                gap: 1,
                                                backgroundColor: 'transparent',
                                            }}
                                        >
                                            {/* Handle per drag */}
                                            <Box {...provided.dragHandleProps} sx={{ cursor: 'grab' }}>
                                                <IconButton sx={{ color: 'text.secondary' }} size="small">
                                                    <Dehaze />
                                                </IconButton>
                                            </Box>

                                            {/* Pattern */}
                                            <TextField
                                                sx={{ flex: 4 }}
                                                fullWidth
                                                label="Pattern"
                                                value={rule.pattern}
                                                size="small"
                                                onChange={(e) => handleChange(index, 'pattern', e.target.value)}
                                            />

                                            {/* Destinazione */}
                                            <TextField
                                                sx={{ flex: 6 }}
                                                fullWidth
                                                label="Destinazione"
                                                value={rule.destination}
                                                size="small"
                                                onChange={(e) => handleChange(index, 'destination', e.target.value)}
                                                InputProps={{
                                                    endAdornment: (
                                                        <InputAdornment position="end">
                                                            <IconButton
                                                                aria-label="seleziona cartella"
                                                                onClick={() => handleFolderPicker(index)}
                                                                edge="end"
                                                                sx ={{ color: 'text.primary' }}
                                                            >
                                                                <FolderOpen />
                                                            </IconButton>
                                                        </InputAdornment>
                                                    ),
                                                }}
                                            />

                                            {/* Delete */}
                                            <IconButton sx={{ color:"text.primary" }} onClick={() => removeRule(index)}>
                                                <Delete />
                                            </IconButton>
                                        </Box>
                                    )}
                                </Draggable>
                            ))}
                            {provided.placeholder}
                        </div>
                    )}
                </Droppable>
            </DragDropContext>

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
