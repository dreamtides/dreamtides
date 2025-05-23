#nullable enable

using Dreamtides.Components;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

namespace Dreamtides
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
        SetTextInternal(text);
        if (animate && text != _originalText)
        {
          // Toggle to restart animation if needed
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }

    public void SetPreviewText(string text, Color color)
    {
      SetOriginalText(_text.text);
      _originalColor = _text.color;
      SetTextInternal(text);
      _text.color = color;
    }

    public void ClearPreviewText()
    {
      if (_originalText != null && _originalColor != null)
      {
        SetTextInternal(_originalText);
        _text.color = _originalColor;
      }
    }

    void SetTextInternal(string text)
    {
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "text must be non-empty");
      _text.text = text;
    }

    void SetOriginalText(string text)
    {
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "original text must be non-empty");
      _originalText = text;
    }
  }
}