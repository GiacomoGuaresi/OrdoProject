import React, { useState, useEffect } from 'react';
import {
  Typography,
  Box,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Tooltip,
} from '@mui/material';

// Funzione helper per estrarre la parte finale del percorso
const getTruncatedPath = (path) => {
  if (!path) return '';
  const parts = path.split(/[/\\]/);
  return parts.pop();
};

const History = () => {
  const [historyData, setHistoryData] = useState([]);

  useEffect(() => {
    window.electronAPI.getHistory().then((data) => {
      setHistoryData(data || []);
    });
  }, []);

  // Gestore per aprire la cartella in Esplora Risorse/Finder
  const handleDestinationClick = (destinationPath) => {
    if (window.electronAPI && window.electronAPI.showInFolder) {
      window.electronAPI.showInFolder(destinationPath);
    } else {
      console.error('electronAPI.showInFolder non è disponibile.');
      // Gestione per ambienti non-Electron, se necessario
    }
  };

  return (
    <Box>
      <Paper>
        <TableContainer>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>
                  <Typography variant="subtitle1" fontWeight="bold">
                    Timestamp
                  </Typography>
                </TableCell>
                <TableCell>
                  <Typography variant="subtitle1" fontWeight="bold">
                    Origine
                  </Typography>
                </TableCell>
                <TableCell>
                  <Typography variant="subtitle1" fontWeight="bold">
                    Destinazione
                  </Typography>
                </TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {historyData.map((item, index) => (
                <TableRow key={index}>
                  <TableCell>{item.timestamp}</TableCell>
                  {/* Cella Origine: troncamento automatico con CSS */}
                  <Tooltip title={item.source} arrow>
                    <TableCell>
                      <Box
                        sx={{
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                          maxWidth: 300, // Puoi regolare questo valore
                        }}
                      >
                        {item.source}
                      </Box>
                    </TableCell>
                  </Tooltip>
                  {/* Cella Destinazione: troncamento e link cliccabile */}
                  <Tooltip title={item.destination} arrow>
                    <TableCell
                      onClick={() => handleDestinationClick(item.destination)}
                      sx={{
                        cursor: 'pointer',
                        '&:hover': {
                          textDecoration: 'underline',
                        },
                      }}
                    >
                      <Box
                        sx={{
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                          maxWidth: 300, // Puoi regolare questo valore
                        }}
                      >
                        {item.destination}
                      </Box>
                    </TableCell>
                  </Tooltip>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
    </Box>
  );
};

export default History;