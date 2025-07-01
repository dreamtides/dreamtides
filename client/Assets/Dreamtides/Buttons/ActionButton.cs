#nullable enable

using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Utils;
using DG.Tweening;
using TMPro;
using UnityEngine;
using Dreamtides.Schema;
using System.Runtime.CompilerServices;
using UnityEngine.Serialization;

[assembly: InternalsVisibleTo("Dreamtides.TestUtils")]
namespace Dreamtides.Buttons
{
  public sealed class ActionButton : Displayable
  {
    [SerializeField] internal Registry _registry = null!;
    [SerializeField] internal SpriteRenderer _background = null!;
    [SerializeField] internal TextMeshPro _text = null!;
    [FormerlySerializedAs("_onPressedMaterial")]
    [SerializeField] internal Material _noOutlineMaterial = null!;
    [SerializeField] internal float _animationDuration = 0.1f;
    [SerializeField] internal float _moveDistance = 1f;
    [SerializeField] internal AudioClip _onClickSound = null!;
    [SerializeField] internal float _fadeDuration = 0.1f;
    [SerializeField] internal Collider _collider = null!;

    Vector3 _originalPosition;
    Color _originalColor;
    Material _originalMaterial = null!;
    Vector3 _originalBackgroundLocalScale;
    Vector3 _originalTextLocalScale;
    Sequence? _currentAnimation;
    GameAction? _action;
    float? _showOnIdleDuration;
    float? _lastSetViewTime;
    ButtonView? _pendingView;
    bool _isVisible = false;
    private bool _isAnimating = false;
    Sequence? _hideSequence;
    float _lastClickTime = -1f;

    private readonly Color _enabledColor = Color.white;
    private readonly Color _disabledColor = new Color(0.7f, 0.7f, 0.7f); // Gray

    protected override void OnStart()
    {
      SaveCurrentValues();
      _originalMaterial = _background.material;
      _collider.enabled = _isVisible;
      UpdateButtonColors();
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
    }

    private void UpdateButtonColors()
    {
      var targetColor = _action != null ? _enabledColor : _disabledColor;
      _text.color = targetColor;
      _background.color = targetColor;
      _background.material = _action != null ? _originalMaterial : _noOutlineMaterial;
    }

    public void SetView(ButtonView? view, Milliseconds? showOnIdleDuration = null)
    {
      _lastSetViewTime = Time.time;

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
          _text.text = "";
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
            _text.text = "";
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
          _text.text = "";
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
          showSequence.Join(_background.transform.DOScale(_originalBackgroundLocalScale, _fadeDuration));
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
      if (_action == null) return;

      SaveCurrentValues();

      _currentAnimation?.Kill();
      Vector3 directionFromCamera = (transform.position - _registry.Layout.MainCamera.transform.position).normalized;
      Vector3 targetPosition = transform.position + directionFromCamera * _moveDistance;
      _currentAnimation = TweenUtils.Sequence("ButtonPressAnimation");
      _currentAnimation.Insert(0, transform.DOMove(targetPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(new Color(0.8f, 0.8f, 0.8f), _animationDuration));
    }

    public override void MouseUp(bool isSameObject)
    {
      if (_action == null) return;

      _currentAnimation?.Kill();
      _currentAnimation = TweenUtils.Sequence("ButtonReleaseAnimation");
      _currentAnimation.Insert(0, transform.DOMove(_originalPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(_originalColor, _animationDuration));
      if (isSameObject)
      {
        var currentTime = Time.time;
        if (currentTime - _lastClickTime >= 0.5f)
        {
          _lastClickTime = currentTime;
          _registry.SoundService.Play(_onClickSound);
          _registry.ActionService.PerformAction(_action);
        }
      }
    }

    private void SaveCurrentValues()
    {
      _originalPosition = transform.position;
      _originalColor = _background.color;
      _originalBackgroundLocalScale = _background.transform.localScale;
      _originalTextLocalScale = _text.transform.localScale;
    }
  }
}