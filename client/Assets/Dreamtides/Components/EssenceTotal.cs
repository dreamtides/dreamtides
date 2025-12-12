#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Layout;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Components
{
  public class EssenceTotal : Displayable
  {
    [SerializeField]
    internal TextMeshProUGUI _text = null!;

    [SerializeField]
    internal TimedEffect _onChange = null!;

    [SerializeField]
    internal bool _landscapeMode;

    [SerializeField]
    internal string? _originalText;

    [SerializeField]
    internal Color _originalColor = Color.white;

    [SerializeField]
    internal bool _activePreview;

    protected override void OnStart()
    {
      _originalColor = _text.color;
      gameObject.SetActive(_landscapeMode == IsLandscape());
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
      _text.text = $"{text}<voffset=-0.04em><size=80%>\ufcec</size></voffset>";
    }

    void SetOriginalText(string text)
    {
      Errors.CheckNotNull(text);
      Errors.CheckArgument(text.Length > 0, "original text must be non-empty");
      _originalText = text;
    }
  }
}
