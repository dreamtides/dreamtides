#nullable enable

using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using TMPro;
using UnityEngine;
using UnityEngine.Serialization;

[assembly: InternalsVisibleTo("Dreamtides.TestUtils")]

namespace Dreamtides.Buttons
{
  public sealed class ActionButton : Displayable
  {
    [SerializeField]
    internal Registry _registry = null!;

    [SerializeField]
    internal SpriteRenderer _background = null!;

    [SerializeField]
    internal TextMeshPro _text = null!;

    [FormerlySerializedAs("_onPressedMaterial")]
    [SerializeField]
    internal Material _noOutlineMaterial = null!;

    [SerializeField]
    internal float _animationDuration = 0.1f;

    [SerializeField]
    internal float _moveDistance = 1f;

    [SerializeField]
    internal AudioClip _onClickSound = null!;

    [SerializeField]
    internal float _fadeDuration = 0.1f;

    [SerializeField]
    internal Collider _collider = null!;

    [SerializeField]
    internal int _debounceDelayMilliseconds = 500;

    [SerializeField]
    Vector3 _originalPosition;

    [SerializeField]
    Color _originalColor;

    [SerializeField]
    Material _originalMaterial = null!;

    [SerializeField]
    Vector3 _originalBackgroundLocalScale;

    [SerializeField]
    Vector3 _originalTextLocalScale;

    Sequence? _currentAnimation;
    Sequence? _hideSequence;
    GameAction? _action;
    float? _showOnIdleDuration;
    float? _lastSetViewTime;
    ButtonView? _pendingView;
    bool _isVisible = false;
    private bool _isAnimating = false;
    float _lastClickTime = -1f;
    bool _ignoreClick = false;
    bool _initialized = false;
    float _lastInteractionTime = 0f;
    float _lastIdlePollTime = 0f;
    bool _hasRestoredAfterIdle = false;

    private readonly Color _enabledColor = Color.white;

    private readonly Color _disabledColor = new Color(0.7f, 0.7f, 0.7f); // Gray

    public SpriteRenderer Background => _background;

    protected override void OnStart()
    {
      _originalMaterial = _background.material;
      _collider.enabled = _isVisible;
      _lastInteractionTime = Time.time;
      _lastIdlePollTime = Time.time;
    }

    private void Update()
    {
      if (_pendingView != null && _showOnIdleDuration.HasValue && _lastSetViewTime.HasValue)
      {
        float elapsedTime = Time.time - _lastSetViewTime.Value;
        if (elapsedTime >= _showOnIdleDuration.Value)
        {
          ApplyView(_pendingView);
          _pendingView = null;
        }
      }
      var now = Time.time;
      if (now - _lastIdlePollTime >= 1f)
      {
        _lastIdlePollTime = now;
        if (
          !_hasRestoredAfterIdle
          && now - _lastInteractionTime >= 1f
          && _initialized
          && !_isAnimating
        )
        {
          RestoreDefaults();
          _hasRestoredAfterIdle = true;
        }
      }
    }

    private void UpdateButtonColors()
    {
      var targetColor = _action != null ? _enabledColor : _disabledColor;
      _text.color = targetColor;
      _background.color = targetColor;
      _background.material = _action != null ? _originalMaterial : _noOutlineMaterial;
    }

    public void SetView(
      ButtonView? view,
      Milliseconds? showOnIdleDuration = null,
      Registry? registry = null
    )
    {
      _registry = registry ?? _registry;
      MaybeInitialize();
      _lastSetViewTime = Time.time;
      MarkInteraction();

      if (view == null)
      {
        _showOnIdleDuration = null;
        _pendingView = null;
        ApplyView(null);
        return;
      }

      _showOnIdleDuration = showOnIdleDuration?.ToSeconds();

      if (_showOnIdleDuration.HasValue && !_isVisible)
      {
        _pendingView = view;
      }
      else
      {
        ApplyView(view);
      }
    }

    private void ApplyView(ButtonView? view)
    {
      if (_isAnimating)
      {
        if (view == null)
        {
          _action = null;
          UpdateButtonColors();
        }
        else
        {
          _text.text = view.Label;
          _action = view.Action?.ToGameAction();
          UpdateButtonColors();
          if (_hideSequence != null)
          {
            _hideSequence.Kill();
            _hideSequence = null;
            _background.transform.localScale = _originalBackgroundLocalScale;
            _text.transform.localScale = _originalTextLocalScale;
            _background.gameObject.SetActive(true);
            _text.gameObject.SetActive(true);
            _isVisible = true;
            _collider.enabled = true;
            _isAnimating = false;
          }
        }
        return;
      }

      if (view == null)
      {
        if (_isVisible)
        {
          _isAnimating = true;
          _hideSequence = TweenUtils.Sequence("ButtonHideAnimation");
          _hideSequence.Join(_background.transform.DOScale(Vector3.zero, _fadeDuration));
          _hideSequence.Join(_text.transform.DOScale(Vector3.zero, _fadeDuration));

          _hideSequence.OnComplete(() =>
          {
            _background.gameObject.SetActive(false);
            _text.gameObject.SetActive(false);
            _action = null;
            _isVisible = false;
            _collider.enabled = false;
            _isAnimating = false;

            _background.transform.localScale = _originalBackgroundLocalScale;
            _text.transform.localScale = _originalTextLocalScale;
            UpdateButtonColors();
          });
        }
        else
        {
          _background.gameObject.SetActive(false);
          _text.gameObject.SetActive(false);
          _action = null;
          _isVisible = false;
          _collider.enabled = false;
          UpdateButtonColors();
        }
      }
      else
      {
        _text.text = view.Label;
        _action = view.Action?.ToGameAction();
        UpdateButtonColors();

        if (!_isVisible)
        {
          _isAnimating = true;
          _background.transform.localScale = Vector3.zero;
          _text.transform.localScale = Vector3.zero;

          _background.gameObject.SetActive(true);
          _text.gameObject.SetActive(true);
          _collider.enabled = true;
          Sequence showSequence = TweenUtils.Sequence("ButtonShowAnimation");
          showSequence.Join(
            _background.transform.DOScale(_originalBackgroundLocalScale, _fadeDuration)
          );
          showSequence.Join(_text.transform.DOScale(_originalTextLocalScale, _fadeDuration));
          showSequence.OnComplete(() => _isAnimating = false);
        }

        _isVisible = true;
        _collider.enabled = true;
      }
    }

    public override bool CanHandleMouseEvents() => true;

    public override void MouseDown()
    {
      MarkInteraction();
      if (_action == null)
        return;
      MaybeInitialize();

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
      Vector3 directionFromCamera = (
        transform.position - _registry.MainCamera.transform.position
      ).normalized;
      Vector3 targetPosition = transform.position + directionFromCamera * _moveDistance;
      _currentAnimation = TweenUtils.Sequence("ButtonPressAnimation");
      _currentAnimation.Insert(0, transform.DOMove(targetPosition, _animationDuration));
      _currentAnimation.Insert(
        0,
        _background.DOColor(new Color(0.8f, 0.8f, 0.8f), _animationDuration)
      );
    }

    public override void MouseUp(bool isSameObject)
    {
      MarkInteraction();
      if (_action == null)
        return;
      if (_ignoreClick)
      {
        _ignoreClick = false;
        return;
      }

      _currentAnimation?.Kill();
      _currentAnimation = TweenUtils.Sequence("ButtonReleaseAnimation");
      _currentAnimation.Insert(0, transform.DOMove(_originalPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(_originalColor, _animationDuration));
      if (isSameObject)
      {
        _registry.ActionService.PerformAction(_action);
      }
    }

    private void MaybeInitialize()
    {
      if (_initialized)
      {
        return;
      }

      _originalPosition = transform.position;
      _originalColor = _background.color;
      _originalBackgroundLocalScale = _background.transform.localScale;
      _originalTextLocalScale = _text.transform.localScale;
      UpdateButtonColors();
      _initialized = true;
    }

    private void RestoreDefaults()
    {
      _currentAnimation?.Kill();
      transform.position = _originalPosition;
      _background.color = _originalColor;
      _background.transform.localScale = _originalBackgroundLocalScale;
      _text.transform.localScale = _originalTextLocalScale;
    }

    private void MarkInteraction()
    {
      _lastInteractionTime = Time.time;
      _hasRestoredAfterIdle = false;
    }
  }
}
