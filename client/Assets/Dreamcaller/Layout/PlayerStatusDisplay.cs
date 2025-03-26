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
      _energy.SetText($"{playerView.Energy} / {playerView.ProducedEnergy} <color=#00838F><voffset=0.1em>\uf7e4</voffset></color>", animate);
      _totalSpark.SetText(playerView.TotalSpark.ToString(), animate);
      SetScore(playerView.Score, animate);
    }

    public void SetScore(long score, bool animate = true)
    {
      _score.SetText($"{score} <voffset=0.1em><size=80%>\uf0a3</size></voffset>", animate);
    }

    protected override Vector3 CalculateObjectPosition(int index, int count) => transform.position;

    protected override Vector3? CalculateObjectRotation(int index, int count) => new(90, _isLandscape ? 90 : 0, 0);

    protected override float? CalculateObjectScale(int index, int count) => 0;
  }
}