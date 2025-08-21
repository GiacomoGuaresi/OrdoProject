import { createTheme } from '@mui/material/styles';

const theme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#FFD86C',
    },
    secondary: {
      main: '#FFD86C',
    },
    background: {
      default: '#202020',
      paper: '#2b2b2b',
    },
    text: {
      primary: '#ffffff',
      secondary: '#cccccc',
    },
  },
  typography: {
    fontFamily: '"Segoe UI", "Roboto", "Arial", sans-serif',
    h6: {
      fontWeight: 600,
      letterSpacing: 0.2,
    },
    button: {
      textTransform: 'none',
      fontWeight: 500,
    },
  },
  shape: {
    borderRadius: 12,
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        html: {
          // Scrollbar per browser Webkit (Chrome, Edge, Safari)
          '&::-webkit-scrollbar': {
            width: '10px',
            height: '10px',
          },
          '&::-webkit-scrollbar-track': {
            background: 'transparent',
            borderRadius: '8px',
          },
          '&::-webkit-scrollbar-thumb': {
            backgroundColor: '#555555',
            borderRadius: '8px',
            border: '2px solid #2b2b2b',
          },
          '&::-webkit-scrollbar-thumb:hover': {
            backgroundColor: '#777777',
          },

          // Scrollbar per Firefox
          scrollbarWidth: 'thin',
          scrollbarColor: '#555555 transparent',
        },
      },
    },

    MuiAppBar: {
      styleOverrides: {
        root: {
          background: 'transparent',
          color: '#ffffff',
          boxShadow: 'none',
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: 16,
          boxShadow: '0 4px 20px rgba(0,0,0,0.05)',
        },
      },
    },
    MuiTabs: {
      styleOverrides: {
        indicator: {
          height: 3,
          borderRadius: 2,
        },
      },
    },
    MuiInputBase: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          backgroundColor: '#404040',
          '& .MuiInputBase-input': {
            padding: '10px 12px',
          },
        },
      },
    },

    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          boxShadow: '0px 1px 2px rgba(0,0,0,0.05)',
          padding: '8px 20px',
          fontSize: '0.95rem',
          transition: 'all 0.2s ease',
        },
      },
      variants: [
        {
          props: { variant: 'buttonPrimary' },
          style: {
            backgroundColor: '#323232',
            color: '#ffffff',
            border: '1px solid #3f3f3f',
            boxShadow: '0 2px 4px rgba(0,0,0,0.3)',
            '&:hover': {
              backgroundColor: '#3f3f3f',
              border: '1px solid #5a5a5a',
            },
            '&:active': {
              backgroundColor: '#2a2a2a',
            },
          },
        },
      ],
    },

    MuiTableRow: {
      styleOverrides: {
        root: {
          '&:hover': {
            backgroundColor: '#3f3f3f',
          },
        },
      },
    },
    MuiTableCell: {
      styleOverrides: {
        root: {
          padding: '12px 16px',
          borderBottom: '1px solid #3f3f3f',
          '&:last-child': {
            paddingRight: 24,
          },
        },
      },
    },

    MuiAccordion: {
      styleOverrides: {
        root: {
          backgroundColor: '#323232',
          border: '1px solid #3f3f3f',
          boxShadow: '0 2px 4px rgba(0,0,0,0.3)',
          color: '#ffffff', // testo chiaro
          borderRadius: 12, // arrotondamenti morbidi
          transition: 'all 0.3s ease', // transizione fluida
          '&:before': {
            display: 'none', // rimuove la linea superiore standard
          },
          '&.Mui-expanded': {
            backgroundColor: '#333333', // colore leggermente più chiaro quando espanso
          },
        },
      },
    },

    MuiAccordionSummary: {
      styleOverrides: {
        root: {
          padding: '0 16px',
          minHeight: 56,
          '& .MuiAccordionSummary-content': {
            margin: '12px 0',
          },
        },
        expandIconWrapper: {
          color: '#FFD86C', // icona a tema Windows 11 gold accent
          transition: 'transform 0.3s ease',
          '&.Mui-expanded': {
            transform: 'rotate(180deg)',
          },
        },
      },
    },

    MuiAccordionDetails: {
      styleOverrides: {
        root: {
          padding: '16px',
          borderTop: '1px solid #3f3f3f',
        },
      },
    },
  },
});

export default theme;
