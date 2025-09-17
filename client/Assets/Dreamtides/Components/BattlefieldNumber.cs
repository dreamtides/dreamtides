#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides
{
  public class BattlefieldNumber : MonoBehaviour
  {
    [SerializeField]
    internal TextMeshPro _text = null!;

    [SerializeField]
    internal TimedEffect _onChange = null!;

    [SerializeField]
    internal string? _originalText;

    [SerializeField]
    internal Color _originalColor = Color.white;

    [SerializeField]
    internal bool _activePreview;

    void Start()
    {
      _originalColor = _text.color;
    }

    public void SetText(string text, bool animate)
    {
      var matchesOriginal = _originalText == text;
      SetOriginalText(text);

      if (_text.text != text && !_activePreview)
      {
        SetTextInternal(text);
        if (animate && !matchesOriginal)
        {
          // Toggle to restart animation if needed
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }

    public void SetPreviewText(string text, Color color)
    {
      SetTextInternal(text);
      _text.color = color;
      _activePreview = true;
    }

    public void ClearPreviewText()
    {
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
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "original text must be non-empty");
      _originalText = text;
    }
  }
}
