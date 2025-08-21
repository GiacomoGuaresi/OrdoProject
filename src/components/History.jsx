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
  IconButton,
} from '@mui/material';
import OpenInNewIcon from '@mui/icons-material/OpenInNew';
import { format } from 'date-fns';

// Funzione helper per formattare timestamp
const formatTimestamp = (timestamp) => {
  try {
    return format(new Date(timestamp), 'dd/MM/yyyy HH:mm:ss');
  } catch {
    return timestamp;
  }
};

const History = () => {
  const [historyData, setHistoryData] = useState([]);

  useEffect(() => {
    window.electronAPI.getHistory().then((data) => {
      setHistoryData(data || []);
    });
  }, []);

  const handleDestinationClick = (destinationPath) => {
    if (window.electronAPI?.showInFolder) {
      window.electronAPI.showInFolder(destinationPath);
    } else {
      console.error('electronAPI.showInFolder non disponibile.');
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
                    Paths
                  </Typography>
                </TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {historyData.map((item, index) => (
                <TableRow key={index}>
                  {/* Timestamp */}
                  <TableCell>{formatTimestamp(item.timestamp)}</TableCell>

                  {/* From / To */}
                  <TableCell>
                    <Box display="flex" flexDirection="column" gap={0.5}>
                      {/* From */}
                      <Box display="flex" alignItems="center">
                        <Typography variant="body2" fontWeight="bold" sx={{ marginRight: 1 }}>
                          From:
                        </Typography>
                        <Tooltip title={item.source} arrow>
                          <Typography
                            variant="body2"
                            sx={{
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                              maxWidth: 800,
                            }}
                          >
                            {item.source}
                          </Typography>
                        </Tooltip>
                      </Box>

                      {/* To */}
                      <Box display="flex" alignItems="center">
                        <Typography variant="body2" fontWeight="bold" sx={{ marginRight: 1 }}>
                          To:
                        </Typography>
                        <Tooltip title={item.destination} arrow>
                          <Box
                            onClick={() => handleDestinationClick(item.destination)}
                            sx={{
                              display: 'flex',
                              alignItems: 'center',
                              cursor: 'pointer',
                              borderBottom: '1px dashed #777777',
                              paddingBottom: '2px', // distanza tra testo e linea
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                              maxWidth: 800,
                              '&:hover': {
                                color: 'primary.main',
                              },
                            }}
                          >
                            <OpenInNewIcon fontSize="small" sx={{ mr: 0.5 }} />
                            <Typography variant="body2" noWrap>
                              {item.destination}
                            </Typography>
                          </Box>
                        </Tooltip>
                      </Box>
                    </Box>
                  </TableCell>
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
