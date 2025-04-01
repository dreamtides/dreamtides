#nullable enable

using DG.Tweening;
using Dreamcaller.Schema;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using TMPro;
using UnityEngine;

namespace Dreamcaller.Buttons
{
  public class ControlledButton : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] CanvasGroup _canvasGroup = null!;
    [SerializeField] bool _showing;
    [SerializeField] TextMeshProUGUI _text = null!;
    ButtonView? _view;

    public CanvasGroup CanvasGroup => _canvasGroup;

    public void OnClick()
    {
      _registry.ActionService.PerformAction(Errors.CheckNotNull(_view?.Action));
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
    }
  }
}