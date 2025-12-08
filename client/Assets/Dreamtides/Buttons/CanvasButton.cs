#nullable enable

using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Buttons
{
  public class CanvasButton : Displayable
  {
    [SerializeField]
    internal CanvasGroup _canvasGroup = null!;

    [SerializeField]
    internal bool _showing;

    [SerializeField]
    internal TextMeshProUGUI _text = null!;

    [SerializeField]
    internal Color _enabledTextColor = Color.white;

    [SerializeField]
    internal Color _disabledTextColor = new(0.55f, 0.55f, 0.55f, 1f);

    ButtonView? _view;
    float _lastClickTime = -1f;

    public CanvasGroup CanvasGroup => _canvasGroup;

    // OnClick to invoke from Unity "Button" component in the editor.
    public void OnClick()
    {
      if (_view?.Action != null)
      {
        var currentTime = Time.time;
        if (currentTime - _lastClickTime >= 0.5f)
        {
          _lastClickTime = currentTime;
          Registry.ActionService.PerformAction(_view.Action?.ToGameAction());
        }
      }
    }

    public void SetView(ButtonView? view)
    {
      _view = view;
      _text.text = view?.Label;

      if (view != null && !_showing)
      {
        _showing = true;
        _canvasGroup.gameObject.SetActive(true);
        TweenUtils.FadeInCanvasGroup(_canvasGroup);
      }

      if (view == null && _showing)
      {
        _showing = false;
        TweenUtils
          .FadeOutCanvasGroup(_canvasGroup)
          .OnComplete(() => _canvasGroup.gameObject.SetActive(false));
      }

      if (view != null)
      {
        _text.color = view.Action?.ToGameAction() != null ? _enabledTextColor : _disabledTextColor;
      }
    }
  }
}
