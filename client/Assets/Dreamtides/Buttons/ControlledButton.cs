#nullable enable

using DG.Tweening;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

namespace Dreamtides.Buttons
{
  public class ControlledButton : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] CanvasGroup _canvasGroup = null!;
    [SerializeField] bool _showing;
    [SerializeField] TextMeshProUGUI _text = null!;
    [SerializeField] Color _enabledTextColor = Color.white;
    [SerializeField] Color _disabledTextColor = new(0.55f, 0.55f, 0.55f, 1f);

    ButtonView? _view;
    float _lastClickTime = -1f;

    public CanvasGroup CanvasGroup => _canvasGroup;

    public void OnClick()
    {
      if (_view?.Action != null)
      {
        var currentTime = Time.time;
        if (currentTime - _lastClickTime >= 0.5f)
        {
          _lastClickTime = currentTime;
          _registry.ActionService.PerformAction(_view.Action?.ToGameAction());
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
        TweenUtils.FadeOutCanvasGroup(_canvasGroup).OnComplete(() => _canvasGroup.gameObject.SetActive(false));
      }

      if (view != null)
      {
        _text.color = view.Action?.ToGameAction() != null ? _enabledTextColor : _disabledTextColor;
      }
    }
  }
}