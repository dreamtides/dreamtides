#nullable enable

using Dreamcaller.Components;
using TMPro;
using UnityEngine;

namespace Dreamcaller
{
  public class BattlefieldNumber : MonoBehaviour
  {
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] TimedEffect _onChange = null!;
    string? _originalText;
    Color _originalColor;

    public void SetText(string text, bool animate)
    {
      if (_text.text != text)
      {
        _text.text = text;
        if (animate)
        {
          // Toggle to restart animation if needed
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }

    public void SetPreviewText(string text, Color color)
    {
      _originalText = _text.text;
      _originalColor = _text.color;
      _text.text = text;
      _text.color = color;
    }

    public void ClearPreviewText()
    {
      if (_originalText != null && _originalColor != null)
      {
        _text.text = _originalText;
        _text.color = _originalColor;
      }
    }
  }
}