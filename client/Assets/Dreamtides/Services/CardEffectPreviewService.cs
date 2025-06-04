#nullable enable

using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Services
{
  public class CardEffectPreviewService : Service
  {
    BattlePreviewView? _current;
    Color _previewTextColor = new Color(0.7f, 0.92f, 0.95f);
    FlexNode? _previousScreenOverlayNode;
    bool _renderedScreenOverlay;

    public void DisplayBattlePreview(BattlePreviewView preview)
    {
      if (_current != preview)
      {
        ClearBattlePreview();
        _current = preview;
        ApplyPreview();
      }
    }

    public void ClearBattlePreview()
    {
      if (_current != null)
      {
        ClearAppliedPreviews();

        if (_renderedScreenOverlay)
        {
          Registry.DocumentService.RenderScreenOverlay(_previousScreenOverlayNode);
          _renderedScreenOverlay = false;
          _previousScreenOverlayNode = null;
        }
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

      if (_current.PreviewMessage != null)
      {
        _previousScreenOverlayNode = Registry.DocumentService.CurrentScreenOverlayNode;
        _renderedScreenOverlay = true;
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