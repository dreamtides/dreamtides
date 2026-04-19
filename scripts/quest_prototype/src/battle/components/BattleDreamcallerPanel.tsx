import type {
  BattleDreamcallerSummary,
  BattleDreamsignSummary,
} from "../types";

export function BattleDreamcallerPanel({
  dreamcaller,
  dreamsigns,
  onClose,
}: {
  dreamcaller: BattleDreamcallerSummary | null;
  dreamsigns: readonly BattleDreamsignSummary[];
  onClose: () => void;
}) {
  return (
    <div className="floating-surface-scrim" onClick={onClose}>
      <div
        className="dreamcaller-panel"
        data-battle-dreamcaller-panel=""
        onClick={(event) => event.stopPropagation()}
      >
        <div className="floating-header">
          <div>
            <p className="eyebrow">Battle Side</p>
            <h3>{dreamcaller?.name ?? "Dreamcaller"}</h3>
            {dreamcaller === null ? null : (
              <p className="floating-subtitle">{dreamcaller.title}</p>
            )}
          </div>
          <button type="button" className="btn ghost sm" onClick={onClose}>
            Close
          </button>
        </div>

        {dreamcaller === null ? (
          <div className="floating-empty">No Dreamcaller is attached to this battle.</div>
        ) : (
          <>
            <div className="dreamcaller-summary">
              <div className="summary-grid">
                <div className="summary-stat">
                  <span className="label">Awakening</span>
                  <span className="value">{String(dreamcaller.awakening)}</span>
                </div>
                <div className="summary-stat">
                  <span className="label">Accent Tide</span>
                  <span className="value">{dreamcaller.accentTide}</span>
                </div>
                <div className="summary-stat">
                  <span className="label">Image</span>
                  <span className="value">#{String(dreamcaller.imageNumber)}</span>
                </div>
              </div>
              <p className="dreamcaller-text">{dreamcaller.renderedText}</p>
            </div>

            <div className="floating-section">
              <h4>Dreamsigns</h4>
              {dreamsigns.length === 0 ? (
                <div className="floating-empty">No active Dreamsigns.</div>
              ) : (
                <div className="dreamsign-list">
                  {dreamsigns.map((dreamsign) => (
                    <div
                      key={`${dreamsign.name}-${dreamsign.tide}-${dreamsign.effectDescription}`}
                      className="dreamsign-card"
                    >
                      <div className="dreamsign-head">
                        <strong>{dreamsign.name}</strong>
                        <span className={`dreamsign-badge ${dreamsign.isBane ? "bane" : ""}`}>
                          {dreamsign.isBane ? "Bane" : dreamsign.tide}
                        </span>
                      </div>
                      <p>{dreamsign.effectDescription}</p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
