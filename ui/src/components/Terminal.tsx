import { useEffect, useRef, useImperativeHandle, forwardRef } from 'react';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { SearchAddon } from 'xterm-addon-search';
import { WebLinksAddon } from 'xterm-addon-web-links';
import 'xterm/css/xterm.css';

export interface TerminalHandle {
  write: (text: string) => void;
  writeln: (text: string) => void;
  clear: () => void;
  search: (query: string) => boolean;
  findNext: () => boolean;
  findPrevious: () => boolean;
  setTheme: (theme: 'dark' | 'light') => void;
}

interface TerminalProps {
  theme?: 'dark' | 'light';
}

const themes = {
  dark: {
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    cursor: '#d4d4d4',
    cursorAccent: '#1e1e1e',
    selectionBackground: '#264f78',
    black: '#000000',
    red: '#cd3131',
    green: '#0dbc79',
    yellow: '#e5e510',
    blue: '#2472c8',
    magenta: '#bc3fbc',
    cyan: '#11a8cd',
    white: '#e5e5e5',
    brightBlack: '#666666',
    brightRed: '#f14c4c',
    brightGreen: '#23d18b',
    brightYellow: '#f5f543',
    brightBlue: '#3b8eea',
    brightMagenta: '#d670d6',
    brightCyan: '#29b8db',
    brightWhite: '#ffffff',
  },
  light: {
    background: '#ffffff',
    foreground: '#000000',
    cursor: '#000000',
    cursorAccent: '#ffffff',
    selectionBackground: '#add6ff',
    black: '#000000',
    red: '#cd3131',
    green: '#008000',
    yellow: '#795e00',
    blue: '#0451a5',
    magenta: '#bc05bc',
    cyan: '#0598bc',
    white: '#555555',
    brightBlack: '#666666',
    brightRed: '#f14c4c',
    brightGreen: '#14ce14',
    brightYellow: '#b5ba00',
    brightBlue: '#0451a5',
    brightMagenta: '#bc05bc',
    brightCyan: '#0598bc',
    brightWhite: '#a5a5a5',
  },
};

export const Terminal = forwardRef<TerminalHandle, TerminalProps>(
  ({ theme = 'dark' }, ref) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const terminalRef = useRef<XTerm | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);
    const searchAddonRef = useRef<SearchAddon | null>(null);
    const lastSearchQuery = useRef<string>('');

    useImperativeHandle(ref, () => ({
      write: (text: string) => terminalRef.current?.write(text),
      writeln: (text: string) => terminalRef.current?.writeln(text),
      clear: () => terminalRef.current?.clear(),
      search: (query: string) => {
        if (!searchAddonRef.current || !query) return false;
        lastSearchQuery.current = query;
        return searchAddonRef.current.findNext(query, {
          incremental: true,
          decorations: {
            matchBackground: '#ffff00',
            matchBorder: '#ffff00',
            matchOverviewRuler: '#ffff00',
            activeMatchBackground: '#ff9900',
            activeMatchBorder: '#ff9900',
            activeMatchColorOverviewRuler: '#ff9900',
          },
        });
      },
      findNext: () => {
        if (!searchAddonRef.current || !lastSearchQuery.current) return false;
        return searchAddonRef.current.findNext(lastSearchQuery.current);
      },
      findPrevious: () => {
        if (!searchAddonRef.current || !lastSearchQuery.current) return false;
        return searchAddonRef.current.findPrevious(lastSearchQuery.current);
      },
      setTheme: (newTheme: 'dark' | 'light') => {
        if (terminalRef.current) {
          terminalRef.current.options.theme = themes[newTheme];
        }
      },
    }));

    useEffect(() => {
      if (!containerRef.current) return;

      const terminal = new XTerm({
        fontFamily:
          '"Noto Sans Mono", "Menlo", "Monaco", "Courier New", monospace',
        fontSize: 13,
        lineHeight: 1.2,
        cursorBlink: false,
        cursorStyle: 'bar',
        scrollback: 10000,
        convertEol: true,
        theme: themes[theme],
        allowProposedApi: true,
      });

      const fitAddon = new FitAddon();
      const searchAddon = new SearchAddon();
      const webLinksAddon = new WebLinksAddon();

      terminal.loadAddon(fitAddon);
      terminal.loadAddon(searchAddon);
      terminal.loadAddon(webLinksAddon);

      terminal.open(containerRef.current);
      fitAddon.fit();

      terminalRef.current = terminal;
      fitAddonRef.current = fitAddon;
      searchAddonRef.current = searchAddon;

      const resizeObserver = new ResizeObserver(() => {
        fitAddon.fit();
      });
      resizeObserver.observe(containerRef.current);

      return () => {
        resizeObserver.disconnect();
        terminal.dispose();
      };
    }, []);

    useEffect(() => {
      if (terminalRef.current) {
        terminalRef.current.options.theme = themes[theme];
      }
    }, [theme]);

    return (
      <div
        ref={containerRef}
        style={{
          height: '100%',
          width: '100%',
          padding: '8px',
          boxSizing: 'border-box',
        }}
      />
    );
  }
);

Terminal.displayName = 'Terminal';
