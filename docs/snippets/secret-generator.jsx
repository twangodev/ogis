export const SecretGenerator = () => {
  const [secret, setSecret] = useState('');
  const [copied, setCopied] = useState(false);

  const generate = () => {
    const bytes = new Uint8Array(32);
    crypto.getRandomValues(bytes);
    setSecret(Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join(''));
    setCopied(false);
  };

  const copy = () => {
    navigator.clipboard.writeText(secret)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      })
      .catch((err) => console.error("Failed to copy:", err));
  };

  useEffect(() => { generate(); }, []);

  const RefreshIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
      <path d="M3 3v5h5"/>
      <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
      <path d="M16 16h5v5"/>
    </svg>
  );

  const CopyIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
    </svg>
  );

  const CheckIcon = () => (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 6 9 17l-5-5"/>
    </svg>
  );

  return (
    <div className="not-prose">
      <div className="flex items-center space-x-2">
        <code className="flex-1 text-sm font-mono text-zinc-950/70 dark:text-white/70 bg-zinc-950/5 dark:bg-white/5 px-3 py-2 rounded-lg overflow-x-auto">
          {secret}
        </code>
        <button
          onClick={generate}
          title="Regenerate"
          className="p-2 rounded-lg bg-zinc-950/10 dark:bg-white/10 text-zinc-950/70 dark:text-white/70 hover:bg-zinc-950/20 dark:hover:bg-white/20 transition-colors cursor-pointer"
        >
          <RefreshIcon />
        </button>
        <button
          onClick={copy}
          title={copied ? "Copied!" : "Copy"}
          className="p-2 rounded-lg bg-zinc-950 dark:bg-white text-white dark:text-zinc-950 hover:bg-zinc-950/80 dark:hover:bg-white/80 transition-colors cursor-pointer"
        >
          {copied ? <CheckIcon /> : <CopyIcon />}
        </button>
      </div>
    </div>
  );
};
