#nullable enable

using DG.Tweening;
using UnityEngine;

public class BackgroundOverlay : MonoBehaviour
{
  [SerializeField] SpriteRenderer _overlay = null!;
  bool _isVisible = false;

  public bool IsVisible => _isVisible;

  public enum DisplayOver
  {
    Battlefield,
    Everything,
  }

  public void Show(DisplayOver displayOver, float alpha, Sequence? sequence)
  {
    if (_isVisible)
    {
      return;
    }

    _isVisible = true;
    _overlay.enabled = true;
    _overlay.color = Color.clear;
    switch (displayOver)
    {
      case DisplayOver.Battlefield:
        _overlay.sortingLayerName = "Default";
        break;
      case DisplayOver.Everything:
        _overlay.sortingLayerName = "Top";
        break;
    }

    if (sequence != null)
    {
      sequence.Insert(0, _overlay.DOBlendableColor(new Color(0, 0, 0, alpha), 0.3f));
    }
    else
    {
      _overlay.color = new Color(0, 0, 0, alpha);
    }
  }

  public void Hide(Sequence? sequence)
  {
    if (!_isVisible)
    {
      return;
    }

    if (sequence != null)
    {
      sequence.Insert(0, _overlay.DOBlendableColor(Color.clear, 0.3f)).AppendCallback(() => _isVisible = false);
    }
    else
    {
      _overlay.color = Color.clear;
    }
  }
}