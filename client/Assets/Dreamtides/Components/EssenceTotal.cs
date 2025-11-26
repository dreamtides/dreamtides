#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class EssenceTotal : MonoBehaviour
  {
    [SerializeField]
    internal TextMeshProUGUI _text = null!;

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

    public void SetValue(string value, bool animate)
    {
      var matchesOriginal = _originalText == value;
      SetOriginalText(value);

      if (_text.text != value && !_activePreview)
      {
        SetTextInternal(value);
        if (animate && !matchesOriginal)
        {
          _onChange.gameObject.SetActive(false);
          _onChange.gameObject.SetActive(true);
        }
      }
    }

    public void SetPreviewValue(string value, Color color)
    {
      SetTextInternal(value);
      _text.color = color;
      _activePreview = true;
    }

    public void ClearPreviewValue()
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
