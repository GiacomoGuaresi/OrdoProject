import React, { useState, useEffect } from 'react';
import {
    Typography,
    Button,
    TextField,
    IconButton,
    Box,
    Paper,
    InputAdornment,
    List,
    ListItem,
    ListItemText,
    Accordion,
    AccordionSummary,
    AccordionDetails,
} from '@mui/material';
import { Delete, Add, Dehaze, FolderOpen, ExpandMore } from '@mui/icons-material';
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

            <Accordion sx={{ mb: 2 }}>
                <AccordionSummary
                    expandIcon={<ExpandMore />}
                    aria-controls="file-rules-content"
                    id="file-rules-header"
                >
                    <Typography variant="subtitle1" sx={{ fontWeight: 'bold' }}>
                        Instructions
                    </Typography>
                </AccordionSummary>

                <AccordionDetails>
                    <Typography variant="body1" gutterBottom>
                        Manage your file organization by setting up rules. Each rule consists of a{' '}
                        <strong>pattern</strong> (a wildcard) and a <strong>destination</strong> folder.
                        The application processes these rules from top to bottom, applying the first rule
                        that matches a file. Files that don't match any rule will be ignored.
                    </Typography>

                    <Typography variant="body1" gutterBottom>
                        You can rearrange the rules by dragging and dropping them using the grabber icon on
                        the left. To delete a rule, simply click the trash can icon.
                    </Typography>

                    <Typography variant="subtitle1" sx={{ fontWeight: 'bold', marginTop: 2, marginBottom: 1 }}>
                        Example Patterns:
                    </Typography>

                    <List dense>
                        <ListItem disablePadding>
                            <ListItemText
                                primary="*.{png,jpg,jpeg}"
                                secondary="Matches all files with the extensions .png, .jpg, or .jpeg."
                            />
                        </ListItem>
                        <ListItem disablePadding>
                            <ListItemText
                                primary="invoice_*.pdf"
                                secondary="Matches all PDF files whose names start with 'invoice_'."
                            />
                        </ListItem>
                        <ListItem disablePadding>
                            <ListItemText
                                primary="*.zip"
                                secondary="Matches all files with the .zip extension."
                            />
                        </ListItem>
                    </List>
                </AccordionDetails>
            </Accordion>


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

                                            {/* Destination */}
                                            <TextField
                                                sx={{ flex: 6 }}
                                                fullWidth
                                                label="Destination"
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
                                                                sx={{ color: 'text.primary' }}
                                                            >
                                                                <FolderOpen />
                                                            </IconButton>
                                                        </InputAdornment>
                                                    ),
                                                }}
                                            />

                                            {/* Delete */}
                                            <IconButton sx={{ color: "text.primary" }} onClick={() => removeRule(index)}>
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

            <Box sx={{ ml: 1, mr: 1, mt: 3, display: 'flex', justifyContent: 'space-between', gap: 2 }}>
                <Button variant="buttonPrimary" startIcon={<Add />} onClick={addRule}>
                    Add
                </Button>
                <Button variant="buttonPrimary" onClick={saveRules}>
                    Save
                </Button>
            </Box>
        </Paper>
    );
};

export default Rules;
