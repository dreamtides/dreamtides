import "./styles/statistics_overlay_styles.css";

export interface StatisticResult {
  label: string;
  counts: { value: string; count: number; color?: string }[];
  total: number;
}

export interface StatisticsOverlayProps {
  visible: boolean;
  results: StatisticResult[];
}

export function StatisticsOverlay({ visible, results }: StatisticsOverlayProps) {
  if (!visible || results.length === 0) {
    return null;
  }

  return (
    <div className="tv-statistics-overlay">
      {results.map((result) => (
        <div key={result.label}>
          <div className="tv-statistics-header">
            <span className="tv-statistics-title">{result.label}</span>
          </div>
          <table className="tv-statistics-table">
            <tbody>
              {result.counts.map((entry) => (
                <tr key={entry.value}>
                  <td>
                    <span className="tv-statistics-value-name">
                      {entry.color && (
                        <span
                          className="tv-statistics-color-swatch"
                          style={{ backgroundColor: entry.color }}
                        />
                      )}
                      {entry.value}
                    </span>
                  </td>
                  <td className="tv-statistics-count">{entry.count}</td>
                </tr>
              ))}
              <tr className="tv-statistics-total-row">
                <td>Total</td>
                <td className="tv-statistics-count">{result.total}</td>
              </tr>
            </tbody>
          </table>
        </div>
      ))}
    </div>
  );
}
