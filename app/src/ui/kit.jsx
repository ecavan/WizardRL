/** Small shared UI pieces. */
import { useEffect } from 'react';

export function Toggle({ on, onChange, label, sub }) {
  return (
    <button type="button" className="flex items-center justify-between gap-3 w-full text-left py-1.5" onClick={() => onChange(!on)} aria-pressed={on}>
      <span className="min-w-0">
        <span className="block text-sm font-semibold text-ink-100">{label}</span>
        {sub && <span className="block text-xs text-ink-300 mt-0.5">{sub}</span>}
      </span>
      <span className={`toggle ${on ? 'on' : ''}`}><i /></span>
    </button>
  );
}

export function Seg({ value, options, onChange, className = '', label }) {
  return (
    <div className={`seg ${className}`} role="group" aria-label={label}>
      {options.map(([v, text]) => (
        <button key={String(v)} type="button" className={value === v ? 'on' : ''} aria-pressed={value === v} onClick={() => onChange(v)}>{text}</button>
      ))}
    </div>
  );
}

export function Spinner({ size = 16 }) {
  return <span className="spin inline-block rounded-full border-2 border-ink-500 border-t-violet-400" style={{ width: size, height: size }} />;
}

export function Sheet({ open, onClose, title, children, wide }) {
  useEffect(() => {
    if (!open) return undefined;
    const k = (e) => e.key === 'Escape' && onClose?.();
    window.addEventListener('keydown', k);
    return () => window.removeEventListener('keydown', k);
  }, [open, onClose]);
  if (!open) return null;
  return (
    <div className="sheet-back fade-up" onPointerDown={(e) => { if (e.target === e.currentTarget) onClose?.(); }}>
      <div className={`sheet panel panel-pad scroll-y ${wide ? '!max-w-3xl' : ''}`} role="dialog" aria-modal="true" aria-label={title}>
        {title && (
          <div className="flex items-center justify-between mb-3 gap-3">
            <h2 className="text-lg font-semibold text-white">{title}</h2>
            {onClose && <button className="btn btn-quiet btn-sm" onClick={onClose}>Close</button>}
          </div>
        )}
        {children}
      </div>
    </div>
  );
}

export const fmtPts = (x) => (x >= 0 ? '+' : '−') + Math.abs(Math.round(x));
export const pct = (x) => `${Math.round(x * 100)}%`;
