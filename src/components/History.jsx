import React from 'react';
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
} from '@mui/material';
import { History as HistoryIcon } from '@mui/icons-material';

const History = () => {
  // Dati di esempio per il mockup
  const historyData = [
    {
      timestamp: '2025-08-20 10:00:00',
      source: 'C:\\Users\\Utente\\Documenti\\report.pdf',
      destination: 'D:\\Backup\\Documenti\\report.pdf',
    },
    {
      timestamp: '2025-08-19 15:30:00',
      source: 'C:\\Users\\Utente\\Immagini\\vacanza.jpg',
      destination: 'E:\\Archivio\\Foto\\vacanza.jpg',
    },
    {
      timestamp: '2025-08-19 12:45:00',
      source: 'C:\\Users\\Utente\\Downloads\\file.zip',
      destination: 'D:\\Backup\\Download\\file.zip',
    },
  ];

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
                  <TableCell>{item.source}</TableCell>
                  <TableCell>{item.destination}</TableCell>
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