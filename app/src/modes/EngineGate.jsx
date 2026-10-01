/** Shown while the engine and the networks load (first visit: about 7 MB; after that, cached). */
import { Spinner } from '../ui/kit.jsx';

export function EngineGate({ eng }) {
  return (
    <div className="page flex items-center justify-center min-h-[50vh]">
      <div className="panel panel-pad w-full max-w-sm text-center space-y-3">
        {eng.error ? (
          <>
            <div className="text-white font-semibold">Couldn't load the bots</div>
            <p className="text-sm text-ink-300">{eng.error}</p>
            <button className="btn btn-primary" onClick={() => location.reload()}>Try again</button>
          </>
        ) : (
          <>
            <div className="flex items-center justify-center gap-2 text-white font-semibold"><Spinner /> Loading the bots</div>
            <div className="bar"><i style={{ width: `${Math.round(eng.progress * 100)}%`, background: 'rgb(var(--heat))' }} /></div>
            <p className="text-xs text-ink-300">The trained network is about 7 MB. It's saved on this device after the first time, so the app works offline.</p>
          </>
        )}
      </div>
    </div>
  );
}
