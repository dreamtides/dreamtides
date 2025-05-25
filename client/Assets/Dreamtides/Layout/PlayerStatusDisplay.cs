#nullable enable

using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class PlayerStatusDisplay : StandardObjectLayout
  {
    [SerializeField] BattlefieldNumber _energy = null!;
    [SerializeField] BattlefieldNumber _score = null!;
    [SerializeField] BattlefieldNumber _totalSpark = null!;
    [SerializeField] GameObject _turnIndicator = null!;
    long _producedEnergy;

    public BattlefieldNumber Energy => _energy;
    public BattlefieldNumber Score => _score;
    public BattlefieldNumber TotalSpark => _totalSpark;

    public void UpdatePlayerView(PlayerView playerView, bool animate)
    {
      SetEnergy(playerView.Energy, playerView.ProducedEnergy, animate);
      SetTotalSpark(playerView.TotalSpark, animate);
      SetScore(playerView.Score, animate);
      _turnIndicator.SetActive(playerView.IsCurrentTurn);
    }

    public void SetEnergy(long currentEnergy, long producedEnergy, bool animate = true)
    {
      _producedEnergy = producedEnergy;
      _energy.SetText(
          $"{currentEnergy}/{producedEnergy} <color=#00838F><voffset=0.1em>\uf7e4</voffset></color>",
          animate);
    }

    public void SetScore(long score, bool animate = true)
    {
      _score.SetText($"{score} <voffset=0.1em><size=80%>\uf0a3</size></voffset>", animate);
    }

    public void SetTotalSpark(long totalSpark, bool animate = true)
    {
      _totalSpark.SetText(totalSpark.ToString(), animate);
    }

    public void ApplyPlayerPreview(PlayerPreviewView preview, Color previewTextColor)
    {
      if (preview.Energy != null)
      {
        string energyText = $"{preview.Energy}/{preview.ProducedEnergy ?? _producedEnergy} " +
          "<color=#00838F><voffset=0.1em>\uf7e4</voffset></color>";
        Energy.SetPreviewText(energyText, previewTextColor);
      }

      if (preview.Score != null)
      {
        string scoreText = $"{preview.Score} <voffset=0.1em><size=80%>\uf0a3</size></voffset>";
        Score.SetPreviewText(scoreText, previewTextColor);
      }

      if (preview.TotalSpark != null)
      {
        TotalSpark.SetPreviewText(preview.TotalSpark.ToString(), previewTextColor);
      }
    }

    public void ClearPlayerPreview()
    {
      Energy.ClearPreviewText();
      Score.ClearPreviewText();
      TotalSpark.ClearPreviewText();
    }

    protected override Vector3 CalculateObjectPosition(int index, int count) => transform.position;

    protected override Vector3? CalculateObjectRotation(int index, int count) => new(90, 0, 0);

    protected override float? CalculateObjectScale(int index, int count) => 0;
  }
}