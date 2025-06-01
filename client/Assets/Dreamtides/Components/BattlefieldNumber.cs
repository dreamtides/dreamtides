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
    [SerializeField] string? _originalText;
    [SerializeField] Color _originalColor = Color.white;
    [SerializeField] bool _activePreview;

    void Start()
    {
      _originalColor = _text.color;
    }

    public void SetText(string text, bool animate)
    {
      Log($"SetText: {text}");
      SetOriginalText(text);

      if (_text.text != text && !_activePreview)
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
      Log($"SetPreviewText: {text}");
      SetTextInternal(text);
      _text.color = color;
      _activePreview = true;
    }

    public void ClearPreviewText()
    {
      Log($"ClearPreviewText: restoring to {_originalText}");
      SetTextInternal(Errors.CheckNotNull(_originalText));
      _text.color = _originalColor;
      _activePreview = false;
    }

    void SetTextInternal(string text)
    {
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "text must be non-empty");
      _text.text = text;
    }

    void SetOriginalText(string text)
    {
      Log($"SetOriginalText: {text}");
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "original text must be non-empty");
      _originalText = text;
    }

    void Log(string message)
    {
      if (name == "UserScore")
      {
        Debug.Log(message);
      }
    }
  }
}