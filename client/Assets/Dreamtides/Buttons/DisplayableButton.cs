#nullable enable

using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;

namespace Dreamtides.Buttons
{
  public sealed class DisplayableButton : Displayable
  {
    [SerializeField]
    internal Registry _registry = null!;

    [SerializeField]
    internal SpriteRenderer _background = null!;

    [SerializeField]
    internal TextMeshPro _text = null!;

    [SerializeField]
    internal Material _noOutlineMaterial = null!;

    [SerializeField]
    internal float _animationDuration = 0.1f;

    [SerializeField]
    internal float _onPressScale = 0.97f;

    [SerializeField]
    internal AudioClip _onClickSound = null!;

    [SerializeField]
    internal Collider _collider = null!;

    [SerializeField]
    internal int _debounceDelayMilliseconds = 500;

    Sequence? _currentAnimation;
    GameAction? _action;
    bool _isVisible = false;
    bool _isAnimating = false;
    bool _isPressed = false;
    float _lastClickTime = -1f;
    bool _ignoreClick = false;
    Material? _materialBeforePress;
    bool _awaitingNextSetView = false;

    private readonly Color _enabledColor = Color.white;
    private readonly Color _disabledColor = new Color(0.7f, 0.7f, 0.7f);

    protected override void OnStart()
    {
      _collider.enabled = _isVisible;
    }

    public void SetView(Registry registry, ButtonView? view)
    {
      _registry = registry;
      ResetPressVisualStateImmediate();
      _awaitingNextSetView = false;

      if (view == null)
      {
        if (_isVisible)
        {
          _background.gameObject.SetActive(false);
          _text.gameObject.SetActive(false);
        }
        _action = null;
        _isVisible = false;
        _collider.enabled = false;
        UpdateButtonColors();
        return;
      }

      _text.text = view.Label;
      _action = view.Action?.ToGameAction();
      UpdateButtonColors();
      _background.gameObject.SetActive(true);
      _text.gameObject.SetActive(true);
      _collider.enabled = true;
      _isVisible = true;
    }

    public override bool CanHandleMouseEvents() => true;

    public override void MouseDown()
    {
      if (_action == null)
      {
        return;
      }

      if (_awaitingNextSetView)
      {
        return;
      }

      if (_isAnimating || _isPressed)
      {
        return;
      }

      var currentTime = Time.time;
      if (currentTime - _lastClickTime < (_debounceDelayMilliseconds / 1000f))
      {
        _registry.LoggingService.LogWarning(
          $"Ignoring click <{_debounceDelayMilliseconds}ms after previous"
        );
        _ignoreClick = true;
        return;
      }

      _registry.SoundService.Play(_onClickSound);
      _lastClickTime = currentTime;

      _currentAnimation?.Kill();
      _isAnimating = true;
      _isPressed = true;
      _materialBeforePress = _background.material;
      _background.material = _noOutlineMaterial;
      _currentAnimation = TweenUtils.Sequence("DisplayableButtonPress");
      _currentAnimation.Insert(0, transform.DOScale(_onPressScale, _animationDuration));
      _currentAnimation.OnComplete(() => _isAnimating = false);
    }

    public override void MouseUp(bool isSameObject)
    {
      if (_action == null)
      {
        return;
      }

      if (_awaitingNextSetView)
      {
        return;
      }

      if (!_isPressed)
      {
        return;
      }

      if (_ignoreClick)
      {
        _ignoreClick = false;
        _isPressed = false;
        return;
      }

      _currentAnimation?.Kill();
      _isAnimating = true;
      _currentAnimation = TweenUtils.Sequence("DisplayableButtonRelease");
      _currentAnimation.Insert(0, transform.DOScale(1f, _animationDuration));
      _currentAnimation.Insert(0, _background.DOFade(0f, _animationDuration));
      _currentAnimation.Insert(0, _text.DOFade(0f, _animationDuration));
      _currentAnimation.OnComplete(() =>
      {
        _isAnimating = false;
        _background.gameObject.SetActive(false);
        _text.gameObject.SetActive(false);
        _collider.enabled = false;
        _isVisible = false;
        _awaitingNextSetView = true;
      });

      if (isSameObject)
      {
        _registry.ActionService.PerformAction(_action);
      }

      _isPressed = false;
    }

    void ResetPressVisualStateImmediate()
    {
      _currentAnimation?.Kill();
      transform.localScale = Vector3.one;
      if (_materialBeforePress != null)
      {
        _background.material = _materialBeforePress;
        _materialBeforePress = null;
      }
      _isAnimating = false;
      _isPressed = false;
    }

    void UpdateButtonColors()
    {
      var targetColor = _action != null ? _enabledColor : _disabledColor;
      _text.color = targetColor;
      _background.color = targetColor;
      _background.material = _action != null ? _background.sharedMaterial : _noOutlineMaterial;
    }
  }
}
