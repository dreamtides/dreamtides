#nullable enable

using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class PlayerStatusDisplay : StandardObjectLayout
  {
    const string EnergyIcon = "\ufa1f";
    const string PointsIcon = "\ufb43";

    [SerializeField]
    BattlefieldNumber _energy = null!;

    [SerializeField]
    BattlefieldNumber _score = null!;

    [SerializeField]
    BattlefieldNumber _totalSpark = null!;

    [SerializeField]
    GameObject _leftTurnIndicator = null!;

    [SerializeField]
    GameObject _rightTurnIndicator = null!;

    [SerializeField]
    Material _imminentVictorySparkBackgroundMaterial = null!;

    [SerializeField]
    GameObject _imminentVictoryIndicator = null!;

    [SerializeField]
    MeshRenderer _characterImage = null!;

    [SerializeField]
    GameObject _testCharacterPrefab = null!;

    [SerializeField]
    StudioType _studioType;

    long _producedEnergy;
    Renderer _sparkBackgroundRenderer = null!;
    Material _sparkBackgroundMaterial = null!;

    public BattlefieldNumber Energy => _energy;
    public BattlefieldNumber Score => _score;
    public BattlefieldNumber TotalSpark => _totalSpark;

    protected override void OnInitialize()
    {
      _sparkBackgroundRenderer = _totalSpark.GetComponent<Renderer>();
      _sparkBackgroundMaterial = _sparkBackgroundRenderer.material;
      Registry.StudioService.CaptureSubject(_studioType, _testCharacterPrefab, _characterImage);
    }

    public void UpdatePlayerView(PlayerView playerView, bool animate)
    {
      SetEnergy(playerView.Energy, playerView.ProducedEnergy, animate);
      SetTotalSpark(playerView.TotalSpark, animate);
      SetScore(playerView.Score, animate);
      _leftTurnIndicator.SetActive(playerView.TurnIndicator == DisplayedTurnIndicator.Left);
      _rightTurnIndicator.SetActive(playerView.TurnIndicator == DisplayedTurnIndicator.Right);
      _sparkBackgroundRenderer.material = playerView.IsVictoryImminent
        ? _imminentVictorySparkBackgroundMaterial
        : _sparkBackgroundMaterial;
      _imminentVictoryIndicator.SetActive(playerView.IsVictoryImminent);
    }

    public void SetEnergy(long currentEnergy, long producedEnergy, bool animate = true)
    {
      _producedEnergy = producedEnergy;
      _energy.SetText(
        $"{currentEnergy}/{producedEnergy}<color=#00838F>{EnergyIcon}</color>",
        animate
      );
    }

    public void SetScore(long score, bool animate = true)
    {
      _score.SetText($"{score}<size=80%>{PointsIcon}</size>", animate);
    }

    public void SetTotalSpark(long totalSpark, bool animate = true)
    {
      _totalSpark.SetText(totalSpark.ToString(), animate);
    }

    public void ApplyPlayerPreview(PlayerPreviewView preview, Color previewTextColor)
    {
      if (preview.Energy != null)
      {
        string energyText =
          $"{preview.Energy}/{preview.ProducedEnergy ?? _producedEnergy}"
          + $"<color=#00838F>{EnergyIcon}</color>";
        Energy.SetPreviewText(energyText, previewTextColor);
      }

      if (preview.Score != null)
      {
        string scoreText = $"{preview.Score}<size=80%>{PointsIcon}</size>";
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

    public override Vector3 CalculateObjectPosition(int index, int count) => transform.position;

    public override Vector3? CalculateObjectRotation(int index, int count) => new(90, 0, 0);

    public override float? CalculateObjectScale(int index, int count) => 0;
  }
}
