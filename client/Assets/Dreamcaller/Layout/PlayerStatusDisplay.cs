#nullable enable

using Dreamcaller.Schema;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class PlayerStatusDisplay : StandardObjectLayout
  {
    [SerializeField] BattlefieldNumber _energy = null!;
    [SerializeField] BattlefieldNumber _score = null!;
    [SerializeField] BattlefieldNumber _totalSpark = null!;
    [SerializeField] bool _isLandscape;

    public BattlefieldNumber Energy => _energy;
    public BattlefieldNumber Score => _score;
    public BattlefieldNumber TotalSpark => _totalSpark;

    public void UpdatePlayerView(PlayerView playerView, bool animate)
    {
      _energy.SetNumber(playerView.Energy, animate);
      _score.SetNumber(playerView.Score, animate);
      _totalSpark.SetNumber(playerView.TotalSpark, animate);
    }

    protected override Vector3 CalculateObjectPosition(int index, int count) => transform.position;

    protected override Vector3? CalculateObjectRotation(int index, int count) => new(90, _isLandscape ? 90 : 0, 0);

    protected override float? CalculateObjectScale(int index, int count) => 0;
  }
}