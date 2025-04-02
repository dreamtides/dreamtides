#nullable enable

using Dreamcaller.Layout;
using Dreamcaller.Services;
using Dreamcaller.Utils;
using DG.Tweening;
using TMPro;
using UnityEngine;
using Dreamcaller.Schema;

namespace Dreamcaller.Buttons
{
  public sealed class PrimaryActionButton : Displayable
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] SpriteRenderer _background = null!;
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] Material _onPressedMaterial = null!;
    [SerializeField] float _animationDuration = 0.1f;
    [SerializeField] float _moveDistance = 1f;
    [SerializeField] AudioClip _onClickSound = null!;
    [SerializeField] float _fadeDuration = 0.1f;

    Vector3 _originalPosition;
    Color _originalColor;
    Material _originalMaterial = null!;
    Vector3 _originalBackgroundLocalScale;
    Vector3 _originalTextLocalScale;
    Sequence? _currentAnimation;
    UserAction? _action;
    float? _showOnIdleDuration;
    float? _lastSetViewTime;
    PrimaryActionButtonView? _pendingView;
    bool _isVisible = false;
    private bool _isAnimating = false;

    protected override void OnStart()
    {
      _originalPosition = transform.position;
      _originalColor = _background.color;
      _originalMaterial = _background.material;
      _originalBackgroundLocalScale = _background.transform.localScale;
      _originalTextLocalScale = _text.transform.localScale;
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

    public void SetView(PrimaryActionButtonView? view)
    {
      _lastSetViewTime = Time.time;

      if (view == null)
      {
        _showOnIdleDuration = null;
        _pendingView = null;
        ApplyView(null);
        return;
      }

      _showOnIdleDuration = view.ShowOnIdleDuration?.ToSeconds();

      if (_showOnIdleDuration.HasValue && !_isVisible)
      {
        _pendingView = view;
      }
      else
      {
        ApplyView(view);
      }
    }

    private void ApplyView(PrimaryActionButtonView? view)
    {
      if (_isAnimating)
      {
        if (view == null)
        {
          _action = null;
          _text.text = "";
        }
        else
        {
          _text.text = view.Label;
          _action = view.Action;
          if (!_isVisible)
          {
            DOTween.Kill("ButtonHideAnimation");
            _background.transform.localScale = _originalBackgroundLocalScale;
            _text.transform.localScale = _originalTextLocalScale;
            _background.gameObject.SetActive(true);
            _text.gameObject.SetActive(true);
            _isVisible = true;
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
          var hideSequence = TweenUtils.Sequence("ButtonHideAnimation");
          hideSequence.Join(_background.transform.DOScale(Vector3.zero, _fadeDuration));
          hideSequence.Join(_text.transform.DOScale(Vector3.zero, _fadeDuration));

          hideSequence.OnComplete(() =>
          {
            _background.gameObject.SetActive(false);
            _text.gameObject.SetActive(false);
            _action = null;
            _text.text = "";
            _isVisible = false;
            _isAnimating = false;

            _background.transform.localScale = _originalBackgroundLocalScale;
            _text.transform.localScale = _originalTextLocalScale;
          });
        }
        else
        {
          _background.gameObject.SetActive(false);
          _text.gameObject.SetActive(false);
          _action = null;
          _text.text = "";
          _isVisible = false;
        }
      }
      else
      {
        _text.text = view.Label;
        _action = view.Action;

        if (!_isVisible)
        {
          _isAnimating = true;
          _background.transform.localScale = Vector3.zero;
          _text.transform.localScale = Vector3.zero;

          _background.gameObject.SetActive(true);
          _text.gameObject.SetActive(true);
          Sequence showSequence = TweenUtils.Sequence("ButtonShowAnimation");
          showSequence.Join(_background.transform.DOScale(_originalBackgroundLocalScale, _fadeDuration));
          showSequence.Join(_text.transform.DOScale(_originalTextLocalScale, _fadeDuration));
          showSequence.OnComplete(() => _isAnimating = false);
        }

        _isVisible = true;
      }
    }

    public override bool CanHandleMouseEvents() => true;

    public override void MouseDown()
    {
      _currentAnimation?.Kill();
      Vector3 directionFromCamera = (transform.position - _registry.Layout.MainCamera.transform.position).normalized;
      Vector3 targetPosition = transform.position + directionFromCamera * _moveDistance;
      _currentAnimation = TweenUtils.Sequence("ButtonPressAnimation");
      _currentAnimation.Insert(0, transform.DOMove(targetPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(new Color(0.8f, 0.8f, 0.8f), _animationDuration));
      _background.material = _onPressedMaterial;
    }

    public override void MouseUp(bool isSameObject)
    {
      _currentAnimation?.Kill();
      _currentAnimation = TweenUtils.Sequence("ButtonReleaseAnimation");
      _currentAnimation.Insert(0, transform.DOMove(_originalPosition, _animationDuration));
      _currentAnimation.Insert(0, _background.DOColor(_originalColor, _animationDuration));
      _currentAnimation.InsertCallback(_animationDuration, () => _background.material = _originalMaterial);
      if (isSameObject)
      {
        _registry.SoundService.Play(_onClickSound);
        _registry.ActionService.PerformAction(_action);
      }
    }
  }
}