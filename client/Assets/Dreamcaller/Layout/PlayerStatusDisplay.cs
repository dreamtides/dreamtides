#nullable enable

using Dreamcaller.Schema;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class PlayerStatusDisplay : StackObjectLayout
  {
    [SerializeField] BattlefieldNumber _energy = null!;
    [SerializeField] BattlefieldNumber _score = null!;
    [SerializeField] BattlefieldNumber _totalSpark = null!;

    public void UpdatePlayerView(PlayerView playerView, bool animate)
    {
      _energy.SetNumber(playerView.Energy, animate);
      _score.SetNumber(playerView.Score, animate);
      _totalSpark.SetNumber(playerView.TotalSpark, animate);
    }
  }
}