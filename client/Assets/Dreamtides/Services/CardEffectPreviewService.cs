#nullable enable

using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Services
{
  public class CardEffectPreviewService : Service
  {
    BattlePreviewView? _current;
    Color _previewTextColor = new Color(0.7f, 0.92f, 0.95f);

    public void DisplayPlayEffectPreview(BattlePreviewView preview)
    {
      if (_current != preview)
      {
        ClearPlayEffectPreview();
        _current = preview;
        ApplyPreview();
      }
    }

    public void ClearPlayEffectPreview()
    {
      if (_current != null)
      {
        ClearAppliedPreviews();
        Registry.DocumentService.RenderScreenOverlay(null);
      }
      _current = null;
    }

    private void ApplyPreview()
    {
      if (_current == null) return;

      foreach (var cardPreview in _current.Cards)
      {
        var card = Registry.LayoutService.GetCard(cardPreview.CardId);
        card.ApplyPreview(cardPreview, _previewTextColor);
      }

      Registry.Layout.UserStatusDisplay.ApplyPlayerPreview(_current.User, _previewTextColor);
      Registry.Layout.EnemyStatusDisplay.ApplyPlayerPreview(_current.Enemy, _previewTextColor);

      // Display preview message if present
      if (_current.PreviewMessage != null)
      {
        Registry.DocumentService.RenderScreenOverlay(_current.PreviewMessage);
      }
    }

    private void ClearAppliedPreviews()
    {
      if (_current == null)
      {
        return;
      }

      foreach (var cardPreview in _current.Cards)
      {
        var card = Registry.LayoutService.GetCard(cardPreview.CardId);
        card.ClearPreview();
      }

      Registry.Layout.UserStatusDisplay.ClearPlayerPreview();
      Registry.Layout.EnemyStatusDisplay.ClearPlayerPreview();
    }
  }
}