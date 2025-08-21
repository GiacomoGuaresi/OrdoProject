// src/App.jsx
import React, { useState } from 'react';
import {
  AppBar,
  Toolbar,
  Typography,
  Tabs,
  Tab,
  Box,
  Container,
  CssBaseline,
} from '@mui/material';
import { ThemeProvider } from '@mui/material/styles';
import theme from './theme'; // importiamo il tema
import Rules from './components/Rules';
import History from './components/History';

function App() {
  const [value, setValue] = useState(0);

  const handleChange = (event, newValue) => {
    setValue(newValue);
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ flexGrow: 1 }}>
        <AppBar position="static">
          <Toolbar>
            <img src="../build/icon.png" alt="Ordo Logo" style={{ height: 40, marginRight: 10 }} />
            <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
              Ordo Project
            </Typography>
            <Tabs
              value={value}
              onChange={handleChange}
              textColor="inherit"
              indicatorColor="secondary"
            >
              <Tab label="Rule" />
              <Tab label="History" />
            </Tabs>
          </Toolbar>
        </AppBar>

        <Container maxWidth="lg" sx={{ mt: 4 }}>
          {value === 0 && <Rules />}
          {value === 1 && <History />}
        </Container>
      </Box>
    </ThemeProvider>
  );
}

export default App;
